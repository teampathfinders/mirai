use std::io::Read;
use std::sync::atomic::Ordering;
use std::time::Instant;

use async_recursion::async_recursion;
use proto::bedrock::{Animate, CacheStatus, ChunkRadiusRequest, ClientToServerHandshake, CommandRequest, CompressionAlgorithm, CONNECTED_PACKET_ID, ConnectedPacket, ContainerClose, FormResponse, Header, Interact, Login, MovePlayer, PlayerAction, RequestAbility, RequestNetworkSettings, ResourcePackClientResponse, SetLocalPlayerAsInitialized, SettingsCommand, TextMessage, TickSync, UpdateSkin, ViolationWarning};
use proto::raknet::{Ack, ConnectedPing, ConnectionRequest, DisconnectNotification, Nak, NewIncomingConnection};

use util::{BinaryRead, MutableBuffer};

use crate::network::{RaknetUserLayer, BedrockUserLayer};
use crate::raknet::{BroadcastPacket, Frame, FrameBatch};
use crate::raknet::DEFAULT_SEND_CONFIG;
use crate::config::SERVER_CONFIG;

impl RaknetUserLayer {
    /// Processes the raw packet coming directly from the network.
    ///
    /// If a packet is an ACK or NACK type, it will be responded to accordingly (using [`Session::process_ack`] and [`Session::process_nak`]).
    /// Frame batches are processed by [`Session::process_frame_batch`].
    pub async fn process_raw_packet(&self, packet: MutableBuffer) -> anyhow::Result<bool> {
        *self.last_update.write() = Instant::now();

        if packet.is_empty() {
            anyhow::bail!("Packet is empty");
        }

        let pk_id = *packet.first().unwrap();
        match pk_id {
            Ack::ID => self.handle_ack(packet.snapshot())?,
            Nak::ID => self.handle_nack(packet.snapshot()).await?,
            _ => self.handle_frame_batch(packet).await?,
        }

        Ok(true)
    }

    /// Processes a broadcasted packet sent by another client connected to the server.
    pub fn handle_broadcast(&self, packet: BroadcastPacket) -> anyhow::Result<()> {
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
    /// * Inserting raknet into the order channels
    /// * Inserting raknet into the compound collector
    /// * Discarding old sequenced frames
    /// * Acknowledging reliable raknet
    async fn handle_frame_batch(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let batch = FrameBatch::deserialize(packet.snapshot())?;
        self
            .batch_number
            .fetch_max(batch.sequence_number, Ordering::SeqCst);

        for frame in batch.frames {
            self.handle_frame(frame, batch.sequence_number).await?;
        }

        Ok(())
    }

    #[async_recursion]
    async fn handle_frame(
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
                self.handle_frame(packet, batch_number).await
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
                    self.handle_frame_body(packet.body).await?;
                }
            }
            return Ok(());
        }

        self.handle_frame_body(frame.body).await
    }

    /// Processes an unencapsulated game packet.
    async fn handle_frame_body(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let packet_id = *packet.first().expect("Game packet buffer was empty");
        match packet_id {
            CONNECTED_PACKET_ID => self.handle_encrypted_frame(packet).await?,
            DisconnectNotification::ID => self.on_disconnect(),
            ConnectionRequest::ID => self.handle_connection_request(packet)?,
            NewIncomingConnection::ID => {
                self.handle_new_incoming_connection(packet)?
            }
            ConnectedPing::ID => self.handle_connected_ping(packet)?,
            id => anyhow::bail!("Invalid Raknet packet ID: {}", id),
        }

        Ok(())
    }
}

impl BedrockUserLayer {   
    async fn handle_encrypted_frame(&self, mut packet: MutableBuffer) -> anyhow::Result<()> {
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
                    self.handle_uncompressed_frame(buffer).await
                }
            }
        } else {
            self.handle_uncompressed_frame(packet).await
        }
    }

    async fn handle_uncompressed_frame(
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
                self.handle_network_settings_request(packet)
            }
            Login::ID => self.handle_login(packet).await,
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
            TextMessage::ID => self.handle_text_message(packet).await,
            SetLocalPlayerAsInitialized::ID => {
                self.process_local_initialized(packet)
            }
            MovePlayer::ID => self.process_move_player(packet).await,
            PlayerAction::ID => self.process_player_action(packet),
            RequestAbility::ID => self.handle_ability_request(packet),
            Animate::ID => self.handle_animation(packet),
            CommandRequest::ID => self.handle_command_request(packet),
            UpdateSkin::ID => self.handle_skin_update(packet),
            SettingsCommand::ID => self.handle_settings_command(packet),
            ContainerClose::ID => self.process_container_close(packet),
            FormResponse::ID => self.handle_form_response(packet),
            TickSync::ID => self.handle_tick_sync(packet),
            id => bail!(Malformed, "Invalid game packet: {id:#04x}"),
        }
    }
}
