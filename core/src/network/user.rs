use std::io::{Read, Write};

use std::sync::{Arc, OnceLock};
use std::sync::atomic::{
    AtomicBool, Ordering, AtomicU32,
};

use anyhow::Context;
use flate2::Compression;
use flate2::write::DeflateEncoder;
use parking_lot::RwLock;
use raknet::{SendConfig, DEFAULT_SEND_CONFIG, RaknetUser, BroadcastPacket};
use tokio::sync::{mpsc, broadcast};
use proto::bedrock::{CommandPermissionLevel, Disconnect, GameMode, PermissionLevel, Skin, ConnectedPacket, CONNECTED_PACKET_ID, CompressionAlgorithm, Packet, Header, RequestNetworkSettings, Login, ClientToServerHandshake, CacheStatus, ResourcePackClientResponse, ViolationWarning, ChunkRadiusRequest, Interact, TextMessage, SetLocalPlayerAsInitialized, MovePlayer, PlayerAction, RequestAbility, Animate, CommandRequest, SettingsCommand, ContainerClose, FormResponseData, TickSync, UpdateSkin, PlayerAuthInput, DisconnectReason};
use proto::crypto::{Encryptor, BedrockIdentity, BedrockClientInfo};
use proto::uuid::Uuid;
use replicator::Replicator;

use tokio::task::JoinHandle;
use util::{Vector, AtomicFlag, Serialize, Deserialize, BinaryWrite, BinaryRead};

use crate::config::SERVER_CONFIG;
use crate::forms::FormSubscriber;
use crate::level::{ChunkViewer, Level};

pub struct BedrockUser {
    pub encryptor: OnceLock<Encryptor>,
    pub identity: OnceLock<BedrockIdentity>,
    pub client_info: OnceLock<BedrockClientInfo>,

    /// Next packet that the server is expecting to receive.
    pub expected: AtomicU32,
    /// Whether compression has been configured.
    pub compressed: AtomicFlag,
    /// Whether the client supports the blob cache.
    pub supports_cache: AtomicBool,
    /// Replication layer.
    pub replicator: Arc<Replicator>,
    pub level: Arc<Level>,
    pub raknet: Arc<RaknetUser>,
    pub player: OnceLock<PlayerData>,
    pub form_subscriber: FormSubscriber,

    pub broadcast: broadcast::Sender<BroadcastPacket>,
    pub job_handle: RwLock<Option<JoinHandle<()>>>
}

impl BedrockUser {
    pub fn new(
        raknet: Arc<RaknetUser>,
        level: Arc<Level>,
        replicator: Arc<Replicator>,
        receiver: mpsc::Receiver<Vec<u8>>,
        broadcast: broadcast::Sender<BroadcastPacket>
    ) -> Arc<Self> {
        let user = Arc::new(Self {
            encryptor: OnceLock::new(),
            identity: OnceLock::new(),
            client_info: OnceLock::new(),
            expected: AtomicU32::new(RequestNetworkSettings::ID),
            compressed: AtomicFlag::new(),
            supports_cache: AtomicBool::new(false),
            
            replicator,
            level,
            raknet,
            player: OnceLock::new(),
            form_subscriber: FormSubscriber::new(),
            
            broadcast,
            job_handle: RwLock::new(None)
        });

        let clone = user.clone();
        let handle = tokio::spawn(async move {
            clone.recv_job(receiver).await;
        });

        *user.job_handle.write() = Some(handle);
        user
    }

    /// Waits for the client to fully disconnect and the server to finish processing.
    pub async fn await_shutdown(self: Arc<Self>) -> anyhow::Result<()> {
        let job_handle = {
            let mut lock = self.job_handle.write();
            lock.take()
        };

        if let Some(job_handle) = job_handle {
            job_handle.await?;
        }

        self.raknet.clone().await_shutdown().await
    }

    async fn recv_job(self: &Arc<Self>, mut receiver: mpsc::Receiver<Vec<u8>>) {
        let mut broadcast = self.broadcast.subscribe();

        let mut should_run = true;
        while should_run {
            tokio::select! {
                packet = receiver.recv() => {            
                    if let Some(packet) = packet {
                        if let Err(err) = self.handle_encrypted_frame(packet).await {
                            tracing::error!("Failed to handle packet: {err:#}");
                        }
                    } else {
                        break
                    }
                },
                packet = broadcast.recv() => {
                    if let Ok(packet) = packet {
                        if let Err(err) = self.handle_broadcast(packet) {
                            tracing::error!("Failed to handle broadcast: {err:#}");
                        }
                    }
                },
                // Use `should_run` variable to trigger one final processing run when shutting down.
                _ = self.raknet.active.cancelled() => should_run = false
            };
        }

        tracing::debug!("Bedrock job exited");
    }

    pub fn handle_broadcast(&self, packet: BroadcastPacket) -> anyhow::Result<()> {
        let should_send = packet.sender.map(|sender| sender != self.raknet.address).unwrap_or(true);
        if should_send {
            self.send_serialized(packet.content.as_ref(), DEFAULT_SEND_CONFIG)?;
        }

        Ok(())
    }

    /// Kicks a player from the server and display the specified message to them.
    #[inline]
    pub fn kick(&self, message: &str) -> anyhow::Result<()> {
        self.kick_with_reason(message, DisconnectReason::Kicked)
    }

    /// Kicks a player from the server and displays the specified message to them.
    /// This also adds a reason to the kick, which is used for telemetry purposes.
    pub fn kick_with_reason(&self, message: &str, reason: DisconnectReason) -> anyhow::Result<()> {
        let disconnect_packet = Disconnect {
            reason, message, hide_message: false
        };
        self.send(disconnect_packet)?;
        self.raknet.active.cancel();

        tracing::info!("{} kicked: {message}", self.name());

        Ok(())
    }

    /// Sends a packet to all initialised sessions including self.
    pub fn broadcast<P: ConnectedPacket + Serialize + Clone>(
        &self,
        packet: P,
    ) -> anyhow::Result<()> {
        self.raknet.broadcast(packet)
    }

    /// Sends a packet to all initialised sessions other than self.
    pub fn broadcast_others<P: ConnectedPacket + Serialize + Clone>(
        &self,
        packet: P,
    ) -> anyhow::Result<()> {
        self.raknet.broadcast_others(packet)
    }

    pub fn handle_disconnect(&self) {
        self.raknet.active.cancel();

        todo!("Disconnect");
    }

    /// Sends a game packet with default settings
    /// (reliable ordered and medium priority)
    pub fn send<T: ConnectedPacket + Serialize>(&self, packet: T) -> anyhow::Result<()> {
        let packet = Packet::new(packet);
        let serialized = packet.serialize()?;

        self.send_serialized(serialized, DEFAULT_SEND_CONFIG)
    }

    /// Sends a game packet with custom reliability and priority
    pub fn send_serialized<B>(&self, packet: B, config: SendConfig) -> anyhow::Result<()>
        where
            B: AsRef<[u8]>
    {
        let mut out;
        if self.compressed.get() {
            let (algorithm, threshold) = {
                let config = SERVER_CONFIG.read();
                (config.compression_algorithm, config.compression_threshold)
            };

            if packet.as_ref().len() > threshold as usize {
                // Compress packet
                match algorithm {
                    CompressionAlgorithm::Snappy => {
                        unimplemented!("Snappy compression");
                    }
                    CompressionAlgorithm::Deflate => {
                        let mut writer = DeflateEncoder::new(
                            vec![CONNECTED_PACKET_ID],
                            Compression::best(),
                        );

                        writer.write_all(packet.as_ref())?;
                        out = writer.finish()?;
                    }
                }
            } else {
                // Also reserve capacity for checksum even if encryption is disabled,
                // preventing allocations.
                out = Vec::with_capacity(1 + packet.as_ref().len() + 8);
                out.write_u8(CONNECTED_PACKET_ID)?;
                out.write_all(packet.as_ref())?;
            }
        } else {
            // Also reserve capacity for checksum even if encryption is disabled,
            // preventing allocations.
            out = Vec::with_capacity(1 + packet.as_ref().len() + 8);
            out.write_u8(CONNECTED_PACKET_ID)?;
            out.write_all(packet.as_ref())?;
        };

        if let Some(encryptor) = self.encryptor.get() {
            encryptor.encrypt(&mut out).context("Failed to encrypt packet")?;
        }

        self.raknet.send_raw_buffer_with_config(out, config);
        Ok(())
    }
  
    async fn handle_encrypted_frame(self: &Arc<Self>, mut packet: Vec<u8>) -> anyhow::Result<()> {
        debug_assert_eq!(packet[0], 0xfe);
        packet.remove(0);

        if let Some(encryptor) = self.encryptor.get() {
            encryptor.decrypt(&mut packet).context("Failed to decrypt packet")?;
        }

        let compression_threshold = SERVER_CONFIG.read().compression_threshold;

        if self.compressed.get()
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

                    self.handle_frame_body(decompressed).await
                }
            }
        } else {
            self.handle_frame_body(packet).await
        }
    }

    async fn handle_frame_body(self: &Arc<Self>, mut packet: Vec<u8>) -> anyhow::Result<()> {
        let start_len = packet.len();
        let mut reader: &[u8] = packet.as_ref();
        let _length = reader.read_var_u32()?;
        let header = Header::deserialize_from(&mut reader)?;

        packet.drain(0..(start_len - reader.remaining()));

        let expected = self.expected();
        if expected != u32::MAX && header.id != expected {
            // Server received an unexpected packet.
            tracing::error!(
                "Client sent unexpected packet while logging in (expected {:#04x}, got {:#04x})",
                expected, header.id
            );
            return self.kick("Unexpected packet")
        }

        match header.id {
            PlayerAuthInput::ID => self.handle_auth_input(packet),
            RequestNetworkSettings::ID => {
                self.handle_network_settings_request(packet)
            }
            Login::ID => self.handle_login(packet).await,
            ClientToServerHandshake::ID => {
                self.handle_client_to_server_handshake(packet)
            }
            CacheStatus::ID => self.handle_cache_status(packet),
            ResourcePackClientResponse::ID => {
                self.handle_resource_client_response(packet)
            }
            ViolationWarning::ID => self.handle_violation_warning(packet),
            ChunkRadiusRequest::ID => self.handle_chunk_radius_request(packet),
            Interact::ID => self.process_interaction(packet),
            TextMessage::ID => self.handle_text_message(packet).await,
            SetLocalPlayerAsInitialized::ID => {
                self.handle_local_initialized(packet)
            }
            MovePlayer::ID => self.process_move_player(packet).await,
            PlayerAction::ID => self.process_player_action(packet),
            RequestAbility::ID => self.handle_ability_request(packet),
            Animate::ID => self.handle_animation(packet),
            CommandRequest::ID => self.handle_command_request(packet),
            UpdateSkin::ID => self.handle_skin_update(packet),
            SettingsCommand::ID => self.handle_settings_command(packet),
            ContainerClose::ID => self.process_container_close(packet),
            FormResponseData::ID => self.handle_form_response(packet),
            TickSync::ID => self.handle_tick_sync(packet),
            id => anyhow::bail!("Invalid game packet: {id:#04x}"),
        }
    }

    /// This function panics if the identity was not set.
    pub fn identity(&self) -> &BedrockIdentity {
        self.identity.get().unwrap()
    }

    /// This function panics if the name was not set.
    pub fn name(&self) -> &str {
        &self.identity().name
    }

    /// This function panics if the XUID was not set.
    pub fn xuid(&self) -> u64 {
        self.identity().xuid
    }

    /// This function panics if the UUID was not set.
    pub fn uuid(&self) -> &Uuid {
        &self.identity().uuid
    }

    /// This function panics if the encryptor was not set.
    pub fn encryptor(&self) -> &Encryptor {
        self.encryptor.get().unwrap()
    }

    /// Returns the next expected packet for this session.
    /// The expected packet will be [`u32::MAX`] if the user is fully
    /// initialized and therefore doesn't follow a strict packet order anymore.
    pub fn expected(&self) -> u32 {
        self.expected.load(Ordering::SeqCst)
    }

    /// Returns whether the user is fully initialized.
    pub fn initialized(&self) -> bool {
        self.expected() == u32::MAX
    }

    /// This functions panic if the player data was not initialized.
    pub fn player(&self) -> &PlayerData {
        self.player.get().unwrap()
    }
}

pub struct PlayerData {
    /// Whether the player's inventory is currently open.
    pub is_inventory_open: AtomicBool,
    /// Position of the player.
    pub position: Vector<f32, 3>,
    /// Rotation of the player.
    /// x and y components are general rotation.
    /// z component is head yaw.
    pub rotation: Vector<f32, 3>,
    /// Game mode.
    pub game_mode: GameMode,
    /// General permission level.
    pub permission_level: PermissionLevel,
    /// Command permission level
    pub command_permission_level: CommandPermissionLevel,
    /// The client's skin.
    pub skin: RwLock<Skin>,
    /// Runtime ID.
    pub runtime_id: u64,
    /// Helper type that loads the chunks around the player.
    pub viewer: ChunkViewer,
}

impl PlayerData {
    pub fn new(skin: Skin, level: Arc<Level>) -> Self {
        Self {
            is_inventory_open: AtomicBool::new(false),
            position: Vector::from([0.0, 50.0, 0.0]),
            rotation: Vector::from([0.0; 3]),
            game_mode: GameMode::Creative,
            permission_level: PermissionLevel::Operator,
            command_permission_level: CommandPermissionLevel::Owner,
            skin: RwLock::new(skin),
            runtime_id: 1,
            viewer: ChunkViewer::new(level)
        }
    }

    pub fn gamemode(&self) -> GameMode {
        self.game_mode
    }

    pub fn runtime_id(&self) -> u64 {
        self.runtime_id
    }

    pub fn permission_level(&self) -> PermissionLevel {
        self.permission_level
    }

    pub fn command_permission_level(&self) -> CommandPermissionLevel {
        self.command_permission_level
    }
}