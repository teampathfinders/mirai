use std::io::Read;
use std::sync::atomic::Ordering;
use std::time::Instant;

use async_recursion::async_recursion;
use bytes::{Buf, Bytes, BytesMut};

use crate::config::SERVER_CONFIG;
use crate::network::header::Header;
use crate::network::packets::cache::CacheStatus;
use crate::network::packets::command::{CommandRequest, SettingsCommand};
use crate::network::packets::login::{
    ChunkRadiusRequest, ClientToServerHandshake, CompressionAlgorithm, Login,
    RequestNetworkSettings, ResourcePackClientResponse,
};
use crate::network::packets::{
    Animate, ConnectedPacket, Interact, MovePlayer, RequestAbility,
    SetLocalPlayerAsInitialized, TextMessage, UpdateSkin, ViolationWarning,
    CONNECTED_PACKET_ID,
};
use crate::network::raknet::packets::{
    Ack, ConnectionRequest, DisconnectNotification, Nak, NewIncomingConnection,
};
use crate::network::raknet::{BroadcastPacket, Frame, FrameBatch};
use crate::network::session::Session;
use common::{bail, nvassert, ReadExtensions, VResult};
use common::{Deserialize, Serialize};

use super::DEFAULT_SEND_CONFIG;
use super::packets::ConnectedPing;

impl Session {
    /// Processes the raw packet coming directly from the network.
    ///
    /// If a packet is an ACK or NACK type, it will be responded to accordingly (using [`Session::handle_ack`] and [`Session::handle_nack`]).
    /// Frame batches are processed by [`Session::handle_frame_batch`].
    pub async fn process_raw_packet(&self, pk: Bytes) -> VResult<bool> {
        *self.raknet.last_update.write() = Instant::now();

        if pk.is_empty() {
            bail!(BadPacket, "Packet is empty");
        }

        let pk_id = *pk.first().unwrap();
        match pk_id {
            Ack::ID => self.handle_ack(pk)?,
            Nak::ID => self.handle_nack(pk).await?,
            _ => self.handle_frame_batch(pk).await?,
        }

        return Ok(true);
    }

    pub fn process_broadcast(&self, pk: BroadcastPacket) -> VResult<()> {
        if let Ok(xuid) = self.get_xuid() {
            if let Some(sender) = pk.sender {
                // Source is self, do not send.
                return Ok(());
            }
        }

        self.send_serialized(pk.content, DEFAULT_SEND_CONFIG)
    }

    /// Processes a batch of frames.
    ///
    /// This performs the actions required by the Raknet reliability layer, such as
    /// * Inserting packets into the order channels
    /// * Inserting packets into the compound collector
    /// * Discarding old sequenced frames
    /// * Acknowledging reliable packets
    async fn handle_frame_batch(&self, pk: Bytes) -> VResult<()> {
        let batch = FrameBatch::deserialize(pk)?;
        self.raknet
            .client_batch_number
            .fetch_max(batch.sequence_number, Ordering::SeqCst);

        for frame in &batch.frames {
            self.handle_frame(frame, batch.sequence_number).await?;
        }

        Ok(())
    }

    #[async_recursion]
    async fn handle_frame(
        &self,
        frame: &Frame,
        batch_number: u32,
    ) -> VResult<()> {
        if frame.reliability.is_sequenced()
            && frame.sequence_index
                < self.raknet.client_batch_number.load(Ordering::SeqCst)
        {
            // Discard packet
            return Ok(());
        }

        if frame.reliability.is_reliable() {
            // Confirm packet
            let mut lock = self.raknet.confirmed_packets.lock();
            lock.push(batch_number);
        }

        if frame.is_compound {
            if let Some(p) =
                self.raknet.compound_collector.insert(frame.clone())
            {
                return self.handle_frame(&p, batch_number).await;
            }

            return Ok(());
        }

        // TODO: Handle errors in processing properly

        // Sequenced implies ordered
        if frame.reliability.is_ordered() || frame.reliability.is_sequenced() {
            // Add packet to order queue
            if let Some(ready) = self.raknet.order_channels
                [frame.order_channel as usize]
                .insert(frame.clone())
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
    async fn handle_unframed_packet(&self, mut pk: Bytes) -> VResult<()> {
        let bytes = pk.as_ref();

        let packet_id = *pk.first().expect("Game packet buffer was empty");
        match packet_id {
            CONNECTED_PACKET_ID => self.handle_game_packet(pk).await?,
            DisconnectNotification::ID => self.flag_for_close(),
            ConnectionRequest::ID => self.handle_connection_request(pk)?,
            NewIncomingConnection::ID => {
                self.handle_new_incoming_connection(pk)?
            }
            ConnectedPing::ID => self.handle_online_ping(pk)?,
            id => bail!(BadPacket, "Invalid Raknet packet ID: {}", id),
        }

        Ok(())
    }

    async fn handle_game_packet(&self, mut pk: Bytes) -> VResult<()> {
        nvassert!(pk.get_u8() == 0xfe);

        // Decrypt packet
        if self.encryptor.initialized() {
            // Safe to unwrap because the encryptor is confirmed to exist.
            let encryptor = self
                .encryptor
                .get()
                .expect("Encryptor was destroyed while it was in use");

            pk = match encryptor.decrypt(pk) {
                Ok(t) => t,
                Err(e) => {
                    return Err(e);
                }
            }
        }

        let compression_enabled =
            self.raknet.compression_enabled.load(Ordering::SeqCst);
        let compression_threshold = SERVER_CONFIG.read().compression_threshold;

        if compression_enabled
            && compression_threshold != 0
            && pk.len() > compression_threshold as usize
        {
            // Packet is compressed
            let decompressed = match SERVER_CONFIG.read().compression_algorithm
            {
                CompressionAlgorithm::Snappy => {
                    unimplemented!("Snappy decompression");
                }
                CompressionAlgorithm::Deflate => {
                    let mut reader =
                        flate2::read::DeflateDecoder::new(pk.as_ref());

                    let mut decompressed = Vec::new();
                    reader.read_to_end(&mut decompressed)?;
                    // .context("Failed to decompress packet using Deflate")?;

                    Bytes::copy_from_slice(decompressed.as_slice())
                }
            };

            self.handle_decompressed_game_packet(decompressed).await
        } else {
            self.handle_decompressed_game_packet(pk).await
        }
    }

    async fn handle_decompressed_game_packet(
        &self,
        mut pk: Bytes,
    ) -> VResult<()> {
        let length = pk.get_var_u32()?;
        let header = Header::deserialize(&mut pk)?;

        match header.id {
            RequestNetworkSettings::ID => {
                self.handle_request_network_settings(pk)
            }
            Login::ID => self.handle_login(pk).await,
            ClientToServerHandshake::ID => {
                self.handle_client_to_server_handshake(pk)
            }
            CacheStatus::ID => self.handle_cache_status(pk),
            ResourcePackClientResponse::ID => {
                self.handle_resource_pack_client_response(pk)
            }
            ViolationWarning::ID => self.handle_violation_warning(pk),
            ChunkRadiusRequest::ID => self.handle_chunk_radius_request(pk),
            Interact::ID => self.handle_interaction(pk),
            TextMessage::ID => self.handle_text_message(pk),
            SetLocalPlayerAsInitialized::ID => {
                self.handle_local_player_initialized(pk)
            }
            MovePlayer::ID => self.handle_move_player(pk),
            RequestAbility::ID => self.handle_ability_request(pk),
            Animate::ID => self.handle_animation(pk),
            CommandRequest::ID => self.handle_command_request(pk),
            UpdateSkin::ID => self.handle_skin_update(pk),
            SettingsCommand::ID => self.handle_settings_command(pk),
            id => bail!(BadPacket, "Invalid game packet: {id:#04x}"),
        }
    }
}
