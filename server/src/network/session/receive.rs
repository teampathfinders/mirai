use std::io::Read;
use std::sync::atomic::Ordering;
use std::time::Instant;

use async_recursion::async_recursion;
use bytes::{Buf, BytesMut};

use crate::config::SERVER_CONFIG;
use crate::network::header::Header;
use crate::network::packets::{
    ChunkRadiusRequest, ClientCacheStatus, ClientToServerHandshake, CompressionAlgorithm,
    GamePacket, Login, MovePlayer, RequestNetworkSettings, ResourcePackClientResponse,
    SetDifficulty, SetLocalPlayerAsInitialized, ViolationWarning, GAME_PACKET_ID, RequestAbility, Animate,
};
use crate::network::packets::{Interact, OnlinePing, TextMessage};
use crate::network::raknet::packets::ConnectionRequest;
use crate::network::raknet::packets::DisconnectNotification;
use crate::network::raknet::packets::NewIncomingConnection;
use crate::network::raknet::packets::{
    Acknowledgement, AcknowledgementRecord, NegativeAcknowledgement,
};
use crate::network::raknet::{Frame, FrameBatch};
use crate::network::session::session::Session;
use crate::network::traits::{Decodable, Encodable};
use common::{bail, vassert, ReadExtensions, VResult};

impl Session {
    /// Processes the raw packet coming directly from the network.
    ///
    /// If a packet is an ACK or NACK type, it will be responded to accordingly (using [`Session::handle_ack`] and [`Session::handle_nack`]).
    /// Frame batches are processed by [`Session::handle_frame_batch`].
    pub async fn handle_raw_packet(&self) -> VResult<()> {
        let packet = tokio::select! {
            _ = self.active.cancelled() => {
                return Ok(())
            },
            task = self.receive_queue.pop() => task
        };
        *self.last_update.write() = Instant::now();

        if packet.is_empty() {
            bail!(BadPacket, "Packet is empty");
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
    async fn handle_frame_batch(&self, task: BytesMut) -> VResult<()> {
        let batch = FrameBatch::decode(task)?;
        self.client_batch_number
            .fetch_max(batch.get_batch_number(), Ordering::SeqCst);

        for frame in batch.get_frames() {
            self.handle_frame(frame, batch.get_batch_number()).await?;
        }

        Ok(())
    }

    #[async_recursion]
    async fn handle_frame(&self, frame: &Frame, batch_number: u32) -> VResult<()> {
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
    async fn handle_unframed_packet(&self, mut task: BytesMut) -> VResult<()> {
        let bytes = task.as_ref();

        let packet_id = *task.first().expect("Game packet buffer was empty");
        match packet_id {
            GAME_PACKET_ID => self.handle_game_packet(task).await?,
            DisconnectNotification::ID => self.flag_for_close(),
            ConnectionRequest::ID => self.handle_connection_request(task)?,
            NewIncomingConnection::ID => self.handle_new_incoming_connection(task)?,
            OnlinePing::ID => self.handle_online_ping(task)?,
            id => bail!(BadPacket, "Invalid Raknet packet ID: {}", id),
        }

        Ok(())
    }

    async fn handle_game_packet(&self, mut packet: BytesMut) -> VResult<()> {
        vassert!(packet.get_u8() == 0xfe);

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
                    return Err(e);

                    // if matches!(e, VError::BadPacket(_)) {
                    //     todo!();
                    // }

                    // TODO
                    // todo!("Disconnect client because of invalid packet");
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
                    reader.read_to_end(&mut decompressed)?;
                    // .context("Failed to decompress packet using Deflate")?;

                    BytesMut::from(decompressed.as_slice())
                }
            };

            self.handle_decompressed_game_packet(decompressed).await
        } else {
            self.handle_decompressed_game_packet(packet).await
        }
    }

    async fn handle_decompressed_game_packet(&self, mut packet: BytesMut) -> VResult<()> {
        let length = packet.get_var_u32()?;
        let header = Header::decode(&mut packet)?;

        match header.id {
            RequestNetworkSettings::ID => self.handle_request_network_settings(packet),
            Login::ID => self.handle_login(packet).await,
            ClientToServerHandshake::ID => self.handle_client_to_server_handshake(packet),
            ClientCacheStatus::ID => self.handle_client_cache_status(packet),
            ResourcePackClientResponse::ID => self.handle_resource_pack_client_response(packet),
            ViolationWarning::ID => self.handle_violation_warning(packet),
            ChunkRadiusRequest::ID => self.handle_chunk_radius_request(packet),
            Interact::ID => self.handle_interaction(packet),
            TextMessage::ID => self.handle_text_message(packet),
            SetLocalPlayerAsInitialized::ID => self.handle_local_player_initialized(packet),
            MovePlayer::ID => self.handle_move_player(packet),
            RequestAbility::ID => self.handle_ability_request(packet),
            Animate::ID => self.handle_animation(packet),
            id => bail!(BadPacket, "Invalid game packet: {:#04x}", id),
        }
    }
}
