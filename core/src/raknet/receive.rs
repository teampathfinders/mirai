use std::io::Read;
use std::sync::atomic::Ordering;
use std::time::{Instant, Duration};

use async_recursion::async_recursion;

use util::{bail, Result};
use util::bytes::{BinaryRead, MutableBuffer};

use crate::network::{CommandRequest, SettingsCommand, ContainerClose};
use crate::network::{
    ChunkRadiusRequest, ClientToServerHandshake, CompressionAlgorithm, Login,
    RequestNetworkSettings, ResourcePackClientResponse,
};
use crate::network::{
    Animate, CONNECTED_PACKET_ID, ConnectedPacket, Interact, MovePlayer,
    RequestAbility, SetLocalPlayerAsInitialized, TextMessage, UpdateSkin,
    ViolationWarning,
};
use crate::raknet::{
    Ack, ConnectionRequest, DisconnectNotification, Nak, NewIncomingConnection,
};
use crate::raknet::{BroadcastPacket, Frame, FrameBatch};
use crate::network::CacheStatus;
use crate::raknet::ConnectedPing;
use crate::raknet::DEFAULT_SEND_CONFIG;
use crate::network::Header;
use crate::config::SERVER_CONFIG;
use crate::network::Session;

use super::{RakNetSession, RakNetMessage};

const RAKNET_MESSAGE_SEND_TIMEOUT: Duration = Duration::from_millis(10);

impl RakNetSession {
    /// Processes the raw packet coming directly from the network.
    ///
    /// If a packet is an ACK or NACK type, it will be responded to accordingly (using [`Session::process_ack`] and [`Session::process_nak`]).
    /// Frame batches are processed by [`Session::process_frame_batch`].
    pub async fn process_raw_packet(&self, packet: MutableBuffer) -> anyhow::Result<bool> {
        *self.last_update.write() = Instant::now();

        if packet.is_empty() {
            bail!(Malformed, "Packet is empty");
        }

        let pk_id = *packet.first().unwrap();
        match pk_id {
            Ack::ID => self.process_ack(packet.snapshot())?,
            Nak::ID => self.process_nak(packet.snapshot()).await?,
            _ => self.process_frame_batch(packet).await?,
        }

        Ok(true)
    }

    /// Processes a broadcast packet.
    ///
    /// If the packet does not contain a sender or the
    /// sender is confirmed to be a different session,
    /// the packet will be sent to the client.
    ///
    /// If the returned boolean is false, the packet was not sent.
    pub fn process_broadcast(&self, packet: BroadcastPacket) -> anyhow::Result<bool> {
        if let Some(source) = packet.source {
            if source == self.session_id {
                return Ok(false)
            }
        }

        self.send_buf(packet.content, DEFAULT_SEND_CONFIG)?;
        Ok(true)
    }

    /// Processes a batch of frames.
    ///
    /// This performs the actions required by the Raknet reliability layer, such as
    /// * Inserting packets into the order channels
    /// * Inserting packets into the compound collector
    /// * Discarding old sequenced frames
    /// * Acknowledging reliable packets
    async fn process_frame_batch(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let batch = FrameBatch::deserialize(packet.snapshot())?;
        self
            .client_batch_number
            .fetch_max(batch.sequence_number, Ordering::SeqCst);

        for frame in batch.frames {
            self.process_frame(frame, batch.sequence_number).await?;
        }

        Ok(())
    }

    /// Processes a single [`Frame`].
    #[async_recursion]
    async fn process_frame(
        &self,
        frame: Frame,
        batch_number: u32,
    ) -> anyhow::Result<()> {
        if frame.reliability.is_sequenced()
            && frame.sequence_index
            < self.client_batch_number.load(Ordering::SeqCst)
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
            let possible_frag = self.compound_collector.insert(frame)?;

            return if let Some(packet) = possible_frag {
                self.process_frame(packet, batch_number).await
            } else {
                // Compound incomplete
                Ok(())
            };
        }

        // Sequenced implies ordered
        if frame.reliability.is_ordered() || frame.reliability.is_sequenced() {
            // Add packet to order queue
            if let Some(ready) = self.order_channels
                [frame.order_channel as usize]
                .insert(frame)
            {
                for packet in ready {
                    self.process_frame_body(packet.body).await?;
                }
            }
            return Ok(());
        }

        self.process_frame_body(frame.body).await
    }

    /// Processes an unencapsulated game packet.
    async fn process_frame_body(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let packet_id = *packet.first().expect("Game packet buffer was empty");
        match packet_id {
            CONNECTED_PACKET_ID => self.process_connected_packet(packet).await?,
            DisconnectNotification::ID => self.on_disconnect().await,
            ConnectionRequest::ID => self.process_connection_request(packet)?,
            NewIncomingConnection::ID => {
                self.process_new_incoming_connection(packet)?
            }
            ConnectedPing::ID => self.process_online_ping(packet)?,
            id => bail!(Malformed, "Invalid Raknet packet ID: {}", id),
        }

        Ok(())
    }

    /// Processes a connected packet.
    ///
    /// This method takes care of decrypting and decompressing
    /// received packets.
    async fn process_connected_packet(&self, mut packet: MutableBuffer) -> anyhow::Result<()> {
        // dbg!(&packet);

        // Remove 0xfe packet ID.
        packet.advance_cursor(1);

        if let Some(encryptor) = self.encryptor.get() {
            match encryptor.decrypt(&mut packet) {
                Ok(_) => (),
                Err(e) => {
                    return Err(e);
                }
            }
        }

        let compression_enabled =
            self.compression_enabled.load(Ordering::SeqCst);

        let compression_threshold = SERVER_CONFIG.read().compression_threshold;

        if compression_enabled
            && compression_threshold != 0
            && packet.len() > compression_threshold as usize
        {
            let alg = SERVER_CONFIG.read().compression_algorithm;

            // Packet is compressed
            match alg {
                CompressionAlgorithm::Snappy => {
                    unimplemented!("Snappy decompression");
                }
                CompressionAlgorithm::Deflate => {
                    let mut reader =
                        flate2::read::DeflateDecoder::new(packet.as_slice());

                    let mut decompressed = Vec::new();
                    reader.read_to_end(&mut decompressed)?;

                    let buffer = MutableBuffer::from(decompressed);
                    self.process_naked_packet(buffer).await
                }
            }
        } else {
            self.process_naked_packet(packet).await
        }
    }

    /// Processes a packet that has been decrypted, decompressed and stripped of the frame header.
    #[inline]
    async fn process_naked_packet(
        &self,
        mut packet: MutableBuffer
    ) -> anyhow::Result<()> {
        let result = self.sender.send_timeout(RakNetMessage::Message(packet), RAKNET_MESSAGE_SEND_TIMEOUT).await;
        if result.is_err() {
            anyhow::bail!("RakNet message channel timed out");
        }

        Ok(())
    }
}
