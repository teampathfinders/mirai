use std::io::Read;

use std::sync::atomic::Ordering;
use std::time::Instant;

use async_recursion::async_recursion;

use crate::SERVER_CONFIG;
use crate::Header;
use crate::CacheStatus;
use crate::{CommandRequest, SettingsCommand};
use crate::{
    ChunkRadiusRequest, ClientToServerHandshake, CompressionAlgorithm, Login,
    RequestNetworkSettings, ResourcePackClientResponse,
};
use crate::{
    Animate, ConnectedPacket, Interact, MovePlayer, RequestAbility,
    SetLocalPlayerAsInitialized, TextMessage, UpdateSkin, ViolationWarning,
    CONNECTED_PACKET_ID,
};
use crate::{
    Ack, ConnectionRequest, DisconnectNotification, Nak, NewIncomingConnection,
};
use crate::{BroadcastPacket, Frame, FrameBatch};
use crate::Session;
use util::{bail, Result};
use util::{Deserialize, Serialize};
use util::bytes::{BinaryReader, MutableBuffer, VarInt};

use crate::DEFAULT_SEND_CONFIG;
use crate::ConnectedPing;

impl Session {
    /// Processes the raw packet coming directly from the network.
    ///
    /// If a packet is an ACK or NACK type, it will be responded to accordingly (using [`Session::handle_ack`] and [`Session::handle_nack`]).
    /// Frame batches are processed by [`Session::handle_frame_batch`].
    pub async fn process_raw_packet(&self, pk: MutableBuffer) -> Result<bool> {
        *self.raknet.last_update.write() = Instant::now();

        if pk.is_empty() {
            bail!(Malformed, "Packet is empty");
        }

        let pk_id = *pk.first().unwrap();
        match pk_id {
            Ack::ID => self.handle_ack(pk.snapshot())?,
            Nak::ID => self.handle_nack(pk.snapshot()).await?,
            _ => self.handle_frame_batch(pk).await?,
        }

        return Ok(true);
    }

    pub fn process_broadcast(&self, pk: BroadcastPacket) -> Result<()> {
        if let Ok(xuid) = self.get_xuid() {
            if let Some(sender) = pk.sender {
                if sender.get() == xuid {
                    // Source is self, do not send.
                    return Ok(());
                }
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
    async fn handle_frame_batch(&self, pk: MutableBuffer) -> Result<()> {
        let batch = FrameBatch::deserialize(pk.snapshot())?;
        self.raknet
            .client_batch_number
            .fetch_max(batch.sequence_number, Ordering::SeqCst);

        for frame in batch.frames {
            self.handle_frame(frame.into(), batch.sequence_number).await?;
        }

        Ok(())
    }

    #[async_recursion]
    async fn handle_frame(
        &self,
        frame: Frame,
        batch_number: u32,
    ) -> Result<()> {
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
            return if let Some(p) =
                self.raknet.compound_collector.insert(frame)
            {
                return self.handle_frame(p.into(), batch_number).await;
            } else {
                // Compound incomplete
                Ok(())
            }
        }

        // Sequenced implies ordered
        if frame.reliability.is_ordered() || frame.reliability.is_sequenced() {
            // Add packet to order queue
            if let Some(ready) = self.raknet.order_channels
                [frame.order_channel as usize]
                .insert(frame)
            {
                for packet in ready {
                    self.handle_unframed_packet(packet.body).await?;
                }
            }
            return Ok(());
        }

        self.handle_unframed_packet(frame.body).await
    }

    /// Processes an unencapsulated game packet.
    async fn handle_unframed_packet(&self, pk: MutableBuffer) -> Result<()> {
        let _bytes = pk.as_ref();

        let packet_id = *pk.first().expect("Game packet buffer was empty");
        match packet_id {
            CONNECTED_PACKET_ID => self.handle_game_packet(pk).await?,
            DisconnectNotification::ID => self.on_disconnect(),
            ConnectionRequest::ID => self.handle_connection_request(pk)?,
            NewIncomingConnection::ID => {
                self.handle_new_incoming_connection(pk)?
            }
            ConnectedPing::ID => self.handle_online_ping(pk)?,
            id => bail!(Malformed, "Invalid Raknet packet ID: {}", id),
        }

        Ok(())
    }

    async fn handle_game_packet(&self, mut pk: MutableBuffer) -> Result<()> {
        // Decrypt packet
        if self.encryptor.initialized() {
            // Safe to unwrap because the encryptor is confirmed to exist.
            let encryptor = self
                .encryptor
                .get()
                .expect("Encryptor was destroyed while it was in use");

            // Remove 0xfe packet ID.
            // let mut snapshot = SharedBuffer::from(&pk.as_slice()[1..]);
            pk.advance_cursor(1);
            match encryptor.decrypt(&mut pk) {
                Ok(_) => (),
                Err(e) => {
                    return Err(e);
                }
            }
        };

        let compression_enabled =
            self.raknet.compression_enabled.load(Ordering::SeqCst);

        let compression_threshold = SERVER_CONFIG.read().compression_threshold;

        if compression_enabled
            && compression_threshold != 0
            && pk.len() > compression_threshold as usize
        {
            let alg = SERVER_CONFIG.read().compression_algorithm;

            // Packet is compressed
            match alg {
                CompressionAlgorithm::Snappy => {
                    unimplemented!("Snappy decompression");
                }
                CompressionAlgorithm::Deflate => {
                    let mut reader =
                        flate2::read::DeflateDecoder::new(pk.as_ref());

                    let mut decompressed = Vec::new();
                    reader.read_to_end(&mut decompressed)?;

                    let buffer = MutableBuffer::from(decompressed);
                    self.handle_decompressed_game_packet(buffer).await
                }
            }
        } else {
            self.handle_decompressed_game_packet(pk).await
        }
    }

    async fn handle_decompressed_game_packet(
        &self,
        mut pk: MutableBuffer,
    ) -> Result<()> {
        pk.advance_cursor(1); // Skip over 0xfe ID.
        let mut snapshot = pk.snapshot();
        let start_len = snapshot.len();
        let _length = snapshot.read_var_u32()?;
        let header = Header::deserialize(&mut snapshot)?;

        // Advance past the header.
        pk.advance_cursor(start_len - snapshot.len());

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
            id => bail!(Malformed, "Invalid game packet: {id:#04x}"),
        }
    }
}
