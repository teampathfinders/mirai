use std::io::Read;
use std::sync::atomic::Ordering;
use std::time::Instant;

use async_recursion::async_recursion;
use bytes::{Buf, BytesMut};

use crate::config::SERVER_CONFIG;
use crate::error::VexResult;
use crate::network::packets::{
    ClientCacheStatus, CompressionAlgorithm, GAME_PACKET_ID, GamePacket, Login,
    RequestNetworkSettings,
};
use crate::network::packets::online_ping::OnlinePing;
use crate::network::raknet::frame::{Frame, FrameBatch};
use crate::network::raknet::header::Header;
use crate::network::raknet::packets::acknowledgements::{Ack, AckRecord, Nack};
use crate::network::raknet::packets::connection_request::ConnectionRequest;
use crate::network::raknet::packets::disconnect::DisconnectNotification;
use crate::network::raknet::packets::new_incoming_connection::NewIncomingConnection;
use crate::network::session::session::Session;
use crate::network::traits::{Decodable, Encodable};
use crate::util::ReadExtensions;
use crate::vex_assert;

impl Session {
    /// Processes the raw packet coming directly from the network.
    ///
    /// If a packet is an ACK or NACK type, it will be responded to accordingly (using [`Session::process_ack`] and [`Session::process_nack`]).
    /// Frame batches are processed by [`Session::handle_frame_batch`].
    pub async fn handle_raw_packet(&self) -> VexResult<()> {
        let task = tokio::select! {
            _ = self.active.cancelled() => {
                return Ok(())
            },
            task = self.receive_queue.pop() => task
        };
        *self.last_update.write() = Instant::now();

        let packet_id = *task.first().unwrap();
        match packet_id {
            Ack::ID => self.process_ack(task),
            Nack::ID => self.process_nack(task).await,
            _ => self.handle_frame_batch(task).await,
        }
    }

    /// Processes a batch of frames.
    ///
    /// This performs the actions required by the Raknet reliability layer, such as
    /// * Inserting packets into the order channels
    /// * Inserting packets into the compound collector
    /// * Discarding old sequenced frames
    /// * Acknowledging reliable packets
    async fn handle_frame_batch(&self, task: BytesMut) -> VexResult<()> {
        let batch = FrameBatch::decode(task)?;
        self.client_batch_number
            .fetch_max(batch.get_batch_number(), Ordering::SeqCst);

        for frame in batch.get_frames() {
            self.handle_frame(frame, batch.get_batch_number()).await?;
        }

        Ok(())
    }

    #[async_recursion]
    async fn handle_frame(&self, frame: &Frame, batch_number: u32) -> VexResult<()> {
        if frame.reliability.is_sequenced()
            && frame.sequence_index < self.client_batch_number.load(Ordering::SeqCst)
        {
            // Discard packet
            return Ok(());
        }

        if frame.reliability.is_reliable() {
            // Send ACK
            let encoded = Ack {
                records: vec![AckRecord::Single(batch_number)],
            }
                .encode();

            let acknowledgement = match encoded {
                Ok(a) => a,
                Err(e) => {
                    tracing::error!("{e}");
                    return Ok(());
                }
            };

            self.ipv4_socket
                .send_to(acknowledgement.as_ref(), self.address)
                .await?;
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
                    self.handle_unframed_packet(packet.body)?;
                }
            }
            return Ok(());
        }

        self.handle_unframed_packet(frame.body.clone())?;
        Ok(())
    }

    /// Processes an unencapsulated game packet.
    fn handle_unframed_packet(&self, mut task: BytesMut) -> VexResult<()> {
        let bytes = task.as_ref();

        let packet_id = *task.first().expect("Game packet buffer was empty");
        match packet_id {
            GAME_PACKET_ID => self.handle_game_packet(task)?,
            DisconnectNotification::ID => {
                tracing::debug!("Session {:X} requested disconnect", self.guid);
                self.flag_for_close();
            }
            ConnectionRequest::ID => self.handle_connection_request(task)?,
            NewIncomingConnection::ID => self.handle_new_incoming_connection(task)?,
            OnlinePing::ID => self.handle_online_ping(task)?,
            id => {
                tracing::info!("ID: {} {:?}", id, task.as_ref());
                todo!("Other game packet IDs")
            }
        }

        Ok(())
    }

    fn handle_game_packet(&self, mut task: BytesMut) -> VexResult<()> {
        tracing::info!("Received {:x?}", task.as_ref());

        vex_assert!(task.get_u8() == 0xfe);

        let compression_threshold = SERVER_CONFIG.read().compression_threshold;
        if compression_threshold != 0 && task.len() > compression_threshold as usize {
            // Packet is compressed
            let decompressed = match SERVER_CONFIG.read().compression_algorithm {
                CompressionAlgorithm::Snappy => {
                    todo!("Snappy decompression");
                }
                CompressionAlgorithm::Deflate => {
                    tracing::info!("{:X?}", task.as_ref());

                    let mut reader = flate2::read::DeflateDecoder::new(task.as_ref());
                    let mut decompressed = Vec::new();
                    reader.read_to_end(&mut decompressed).unwrap();
                    BytesMut::from(decompressed.as_ref() as &[u8])
                }
            };

            self.handle_decompressed_game_packet(decompressed)
        } else {
            self.handle_decompressed_game_packet(task)
        }
    }

    fn handle_decompressed_game_packet(&self, mut task: BytesMut) -> VexResult<()> {
        let length = task.get_var_u32()?;
        let header = Header::decode(&mut task)?;

        match header.id {
            RequestNetworkSettings::ID => self.handle_request_network_settings(task),
            Login::ID => self.handle_login(task),
            ClientCacheStatus::ID => self.handle_client_cache_status(task),
            _ => todo!(),
        }
    }
}
