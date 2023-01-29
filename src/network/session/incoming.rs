use std::io::Read;
use std::sync::atomic::Ordering;
use std::time::Instant;

use anyhow::{bail, Context};
use async_recursion::async_recursion;
use bytes::{Buf, BytesMut};

use crate::config::SERVER_CONFIG;
use crate::network::header::Header;
use crate::network::packets::{
    ClientCacheStatus, CompressionAlgorithm, GAME_PACKET_ID, GamePacket, Login,
    RequestNetworkSettings,
};
use crate::network::packets::OnlinePing;
use crate::network::raknet::{Frame, FrameBatch};
use crate::network::raknet::packets::{
    Acknowledgement, AcknowledgementRecord, NegativeAcknowledgement,
};
use crate::network::raknet::packets::ConnectionRequest;
use crate::network::raknet::packets::DisconnectNotification;
use crate::network::raknet::packets::NewIncomingConnection;
use crate::network::session::session::Session;
use crate::network::traits::{Decodable, Encodable};
use crate::util::ReadExtensions;
use crate::vex_assert;

impl Session {
    /// Processes the raw packet coming directly from the network.
    ///
    /// If a packet is an ACK or NACK type, it will be responded to accordingly (using [`Session::handle_ack`] and [`Session::handle_nack`]).
    /// Frame batches are processed by [`Session::handle_frame_batch`].
    pub async fn handle_raw_packet(&self) -> anyhow::Result<()> {
        let packet = tokio::select! {
            _ = self.active.cancelled() => {
                return Ok(())
            },
            task = self.receive_queue.pop() => task
        };
        *self.last_update.write() = Instant::now();

        if packet.is_empty() {
            bail!("Packet must not be empty");
        }

        let packet_id = *packet.first().unwrap();
        match packet_id {
            Acknowledgement::ID => self.handle_ack(packet),
            NegativeAcknowledgement::ID => self.handle_nack(packet).await,
            _ => self.handle_frame_batch(packet).await,
        }
    }

    /// Processes a batch of frames.
    ///
    /// This performs the actions required by the Raknet reliability layer, such as
    /// * Inserting packets into the order channels
    /// * Inserting packets into the compound collector
    /// * Discarding old sequenced frames
    /// * Acknowledging reliable packets
    async fn handle_frame_batch(&self, task: BytesMut) -> anyhow::Result<()> {
        let batch = FrameBatch::decode(task)?;
        self.client_batch_number
            .fetch_max(batch.get_batch_number(), Ordering::SeqCst);

        for frame in batch.get_frames() {
            self.handle_frame(frame, batch.get_batch_number()).await?;
        }

        Ok(())
    }

    #[async_recursion]
    async fn handle_frame(&self, frame: &Frame, batch_number: u32) -> anyhow::Result<()> {
        if frame.reliability.is_sequenced()
            && frame.sequence_index < self.client_batch_number.load(Ordering::SeqCst)
        {
            // Discard packet
            return Ok(());
        }

        if frame.reliability.is_reliable() {
            // Confirm packet
            let mut lock = self.confirmed_packets.lock();
            lock.push(batch_number);
        }

        if frame.is_compound {
            if let Some(p) = self.compound_collector.insert(frame.clone()) {
                return self.handle_frame(&p, batch_number).await;
            }

            return Ok(());
        }

        // TODO: Handle errors in processing properly

        // Sequenced implies ordered
        if frame.reliability.is_ordered() || frame.reliability.is_sequenced() {
            // Add packet to order queue
            if let Some(ready) =
                self.order_channels[frame.order_channel as usize].insert(frame.clone())
            {
                for packet in ready {
                    self.handle_unframed_packet(packet.body).await?;
                }
            }
            return Ok(());
        }

        self.handle_unframed_packet(frame.body.clone()).await?;
        Ok(())
    }

    /// Processes an unencapsulated game packet.
    async fn handle_unframed_packet(&self, mut task: BytesMut) -> anyhow::Result<()> {
        let bytes = task.as_ref();

        let packet_id = *task.first().expect("Game packet buffer was empty");
        match packet_id {
            GAME_PACKET_ID => self.handle_game_packet(task).await?,
            DisconnectNotification::ID => self.disconnect(),
            ConnectionRequest::ID => self.handle_connection_request(task)?,
            NewIncomingConnection::ID => self.handle_new_incoming_connection(task)?,
            OnlinePing::ID => self.handle_online_ping(task)?,
            id => bail!("Invalid Raknet packet ID: {}", id),
        }

        Ok(())
    }

    async fn handle_game_packet(&self, mut packet: BytesMut) -> anyhow::Result<()> {
        vex_assert!(packet.get_u8() == 0xfe);

        // Decrypt packet
        if self.encryptor.initialized() {
            // Safe to unwrap because the encryptor is confirmed to exist.
            let encryptor = self
                .encryptor
                .get()
                .expect("Encryptor was destroyed while it was in use");

            packet = match encryptor.decrypt(packet) {
                Ok(t) => t,
                Err(e) => {
                    todo!("Disconnect client because of invalid packet");
                }
            }
        }

        let compression_enabled = self.compression_enabled.load(Ordering::SeqCst);
        let compression_threshold = SERVER_CONFIG.read().compression_threshold;

        if compression_enabled
            && compression_threshold != 0
            && packet.len() > compression_threshold as usize
        {
            // Packet is compressed
            let decompressed = match SERVER_CONFIG.read().compression_algorithm {
                CompressionAlgorithm::Snappy => {
                    todo!("Snappy decompression");
                }
                CompressionAlgorithm::Deflate => {
                    let mut reader = flate2::read::DeflateDecoder::new(packet.as_ref());
                    let mut decompressed = Vec::new();
                    reader
                        .read_to_end(&mut decompressed)
                        .context("Failed to decompress packet using Deflate")?;

                    BytesMut::from(decompressed.as_slice())
                }
            };

            self.handle_decompressed_game_packet(decompressed).await
        } else {
            self.handle_decompressed_game_packet(packet).await
        }
    }

    async fn handle_decompressed_game_packet(&self, mut task: BytesMut) -> anyhow::Result<()> {
        let length = task.get_var_u32()?;
        let header = Header::decode(&mut task)?;

        match header.id {
            RequestNetworkSettings::ID => self.handle_request_network_settings(task),
            Login::ID => self.handle_login(task).await,
            ClientCacheStatus::ID => self.handle_client_cache_status(task),
            _ => todo!(),
        }
    }
}
