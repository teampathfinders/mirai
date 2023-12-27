use std::io::Read;
use std::sync::atomic::Ordering;
use std::time::Instant;

use async_recursion::async_recursion;

use util::{bail, Result};
use util::bytes::{BinaryRead, MutableBuffer};

use crate::network::{CommandRequest, SettingsCommand, ContainerClose, TickSync, FormResponse, PlayerAction};
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

impl Session {
    /// Processes the raw packet coming directly from the network.
    ///
    /// If a packet is an ACK or NACK type, it will be responded to accordingly (using [`Session::process_ack`] and [`Session::process_nak`]).
    /// Frame batches are processed by [`Session::process_frame_batch`].
    pub async fn process_raw_packet(&self, packet: MutableBuffer) -> anyhow::Result<bool> {
        *self.raknet.last_update.write() = Instant::now();

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

    pub fn process_broadcast(&self, packet: BroadcastPacket) -> anyhow::Result<()> {
        if let Ok(xuid) = self.get_xuid() {
            if let Some(sender) = packet.sender {
                if sender.get() == xuid {
                    // Source is self, do not send.
                    return Ok(());
                }
            }
        }

        self.send_serialized(packet.content, DEFAULT_SEND_CONFIG)
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
        self.raknet
            .client_batch_number
            .fetch_max(batch.sequence_number, Ordering::SeqCst);

        for frame in batch.frames {
            self.process_frame(frame, batch.sequence_number).await?;
        }

        Ok(())
    }

    #[async_recursion]
    async fn process_frame(
        &self,
        frame: Frame,
        batch_number: u32,
    ) -> anyhow::Result<()> {
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
            let possible_frag = self.raknet.compound_collector.insert(frame)?;

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
            if let Some(ready) = self.raknet.order_channels
                [frame.order_channel as usize]
                .insert(frame)
            {
                for packet in ready {
                    self.process_frame_data(packet.body).await?;
                }
            }
            return Ok(());
        }

        self.process_frame_data(frame.body).await
    }

    /// Processes an unencapsulated game packet.
    async fn process_frame_data(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let packet_id = *packet.first().expect("Game packet buffer was empty");
        match packet_id {
            CONNECTED_PACKET_ID => self.process_connected_packet(packet).await?,
            DisconnectNotification::ID => self.on_disconnect(),
            ConnectionRequest::ID => self.process_connection_request(packet)?,
            NewIncomingConnection::ID => {
                self.process_new_incoming_connection(packet)?
            }
            ConnectedPing::ID => self.process_online_ping(packet)?,
            id => bail!(Malformed, "Invalid Raknet packet ID: {}", id),
        }

        Ok(())
    }

    async fn process_connected_packet(&self, mut packet: MutableBuffer) -> anyhow::Result<()> {
        // dbg!(&packet);

        // Remove 0xfe packet ID.
        packet.advance_cursor(1);

        // Decrypt packet
        if self.encryptor.initialized() {
            // Safe to unwrap because the encryptor is confirmed to exist.
            let encryptor = self
                .encryptor
                .get()
                .expect("Encryptor was destroyed while it was in use");

            match encryptor.decrypt(&mut packet) {
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
                    self.process_decompressed_packet(buffer).await
                }
            }
        } else {
            self.process_decompressed_packet(packet).await
        }
    }

    async fn process_decompressed_packet(
        &self,
        mut packet: MutableBuffer,
    ) -> anyhow::Result<()> {
        let mut snapshot = packet.snapshot();
        let start_len = snapshot.len();
        let _length = snapshot.read_var_u32()?;

        let header = Header::deserialize(&mut snapshot)?;

        // Advance past the header.
        packet.advance_cursor(start_len - snapshot.len());

        // dbg!(&header);
        match header.id {
            RequestNetworkSettings::ID => {
                self.process_network_settings_request(packet)
            }
            Login::ID => self.process_login(packet).await,
            ClientToServerHandshake::ID => {
                self.process_cts_handshake(packet)
            }
            CacheStatus::ID => self.process_cache_status(packet),
            ResourcePackClientResponse::ID => {
                self.process_pack_client_response(packet)
            }
            ViolationWarning::ID => self.process_violation_warning(packet),
            ChunkRadiusRequest::ID => self.process_radius_request(packet),
            Interact::ID => self.process_interaction(packet),
            TextMessage::ID => self.process_text_message(packet),
            SetLocalPlayerAsInitialized::ID => {
                self.process_local_initialized(packet)
            }
            MovePlayer::ID => self.process_move_player(packet),
            PlayerAction::ID => self.process_player_action(packet),
            RequestAbility::ID => self.process_ability_request(packet),
            Animate::ID => self.process_animation(packet),
            CommandRequest::ID => self.process_command_request(packet),
            UpdateSkin::ID => self.process_skin_update(packet),
            SettingsCommand::ID => self.process_settings_command(packet),
            ContainerClose::ID => self.process_container_close(packet),
            FormResponse::ID => self.process_form_response(packet),
            TickSync::ID => self.process_tick_sync(packet),
            id => bail!(Malformed, "Invalid game packet: {id:#04x}"),
        }
    }
}
