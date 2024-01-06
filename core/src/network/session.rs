use std::net::SocketAddr;

use std::sync::Arc;
use std::sync::atomic::{
    AtomicBool, AtomicU64, Ordering, AtomicU32, AtomicU16,
};
use std::time::Instant;
use anyhow::anyhow;

use parking_lot::{Mutex, RwLock};
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, mpsc, OnceCell};
use tokio_util::sync::CancellationToken;
use proto::bedrock::{CommandPermissionLevel, DeviceOS, Disconnect, GameMode, PermissionLevel, Skin, ConnectedPacket};
use proto::crypto::{Encryptor, IdentityData, UserData};
use proto::uuid::Uuid;
use replicator::Replicator;

use util::{error, Vector, AtomicFlag, Serialize};
use util::MutableBuffer;

use crate::raknet::{BroadcastPacket, RaknetData, Recovery, SendQueues, Compounds, OrderChannel};
use crate::level::{ChunkViewer, LevelManager};

static RUNTIME_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

impl RaknetUser {
    pub fn handle_disconnect(&self) {
        self.active.cancel();
    }
}

pub struct BedrockUserLayer {
    pub compressed: AtomicFlag,
    pub encryptor: Encryptor,
    pub xuid: u64,
    pub name: String,

    pub raknet: RaknetUser,
    pub receiver: mpsc::Receiver<MutableBuffer>
}

impl BedrockUserLayer {
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
    pub fn send_serialized<B>(&self, packet: B, config: PacketConfig) -> anyhow::Result<()>
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
                        out = MutableBuffer::from(writer.finish()?)
                    }
                }
            } else {
                // Also reserve capacity for checksum even if encryption is disabled,
                // preventing allocations.
                out = MutableBuffer::with_capacity(1 + packet.as_ref().len() + 8);
                out.write_u8(CONNECTED_PACKET_ID)?;
                out.write_all(packet.as_ref())?;
            }
        } else {
            // Also reserve capacity for checksum even if encryption is disabled,
            // preventing allocations.
            out = MutableBuffer::with_capacity(1 + packet.as_ref().len() + 8);
            out.write_u8(CONNECTED_PACKET_ID)?;
            out.write_all(packet.as_ref())?;
        };

        self.encryptor.encrypt(&mut out)?;

        self.raknet.send_raw_buffer_with_config(out, config);
        Ok(())
    }
  
    async fn handle_encrypted_frame(&self, mut packet: MutableBuffer) -> anyhow::Result<()> {
        // Remove 0xfe packet ID.
        packet.advance_cursor(1);

        self.encryptor.decrypt(&mut packet)?;

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

                    let buffer = MutableBuffer::from(decompressed);
                    self.handle_frame_body(buffer).await
                }
            }
        } else {
            self.handle_frame_body(packet).await
        }
    }

    async fn handle_frame_body(
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
            FormResponse::ID => self.handle_form_response(packet),
            TickSync::ID => self.handle_tick_sync(packet),
            id => anyhow::bail!("Invalid game packet: {id:#04x}"),
        }
    }
}


// pub struct PlayerData {
//     /// Whether the player's inventory is currently open.
//     pub is_inventory_open: bool,
//     /// Position of the player.
//     pub position: Vector<f32, 3>,
//     /// Rotation of the player.
//     /// x and y components are general rotation.
//     /// z component is head yaw.
//     pub rotation: Vector<f32, 3>,
//     /// Game mode.
//     pub game_mode: GameMode,
//     /// General permission level.
//     pub permission_level: PermissionLevel,
//     /// Command permission level
//     pub command_permission_level: CommandPermissionLevel,
//     /// The client's skin.
//     pub skin: Option<Skin>,
//     /// Runtime ID.
//     pub runtime_id: u64,
//     /// Helper type that loads the chunks around the player.
//     pub viewer: ChunkViewer,
// }

// /// Sessions directly correspond to clients connected to the server.
// ///
// /// Anything that has to do with specific clients must be communicated with their associated sessions.
// /// The server does not interact with clients directly, everything is done through these sessions.
// pub struct User {
//     /// Identity data such as XUID and display name.
//     pub identity: OnceCell<IdentityData>,
//     /// Extra user data, such as device OS and language.
//     pub user_data: OnceCell<UserData>,
//     /// Used to encrypt and decrypt raknet.
//     pub encryptor: OnceCell<Encryptor>,
//     /// Whether the client supports the chunk cache.
//     pub cache_support: OnceCell<bool>,
//     /// Whether the client has fully been initialised.
//     /// This is set to true after receiving the [`SetLocalPlayerAsInitialized`](crate::network::SetLocalPlayerAsInitialized) packet
//     pub initialized: AtomicBool,
//     /// Manages entire world.
//     pub level: Arc<LevelManager>,
//     /// Sends raknet into the broadcasting channel.
//     pub broadcast: broadcast::Sender<BroadcastPacket>,
//     /// Indicates whether this session is active.
//     pub active: CancellationToken,
//     /// Current tick of this session, this is increased every [`TICK_INTERVAL`](crate::level::TICK_INTERVAL).
//     pub current_tick: AtomicU64,
//     /// Minecraft-specific data.
//     pub player: RwLock<PlayerData>,
//     /// Raknet-specific data.
//     pub raknet: RaknetData,
//     pub replicator: Arc<Replicator>
// }

// impl User {
//     /// Creates a new session.
//     pub fn new(
//         broadcast: broadcast::Sender<BroadcastPacket>,
//         receiver: mpsc::Receiver<MutableBuffer>,
//         level_manager: Arc<LevelManager>,
//         replicator: Arc<Replicator>,
//         ipv4_socket: Arc<UdpSocket>,
//         address: SocketAddr,
//         mtu: u16,
//         guid: u64,
//     ) -> Arc<Self> {
//         let session = Arc::new(Self {
//             identity: OnceCell::new(),
//             user_data: OnceCell::new(),
//             encryptor: OnceCell::new(),
//             cache_support: OnceCell::new(),
//             initialized: AtomicBool::new(false),
//             player: RwLock::new(PlayerData {
//                 is_inventory_open: false,
//                 position: Vector::from([0.0, 0.0, 0.0]),
//                 rotation: Vector::from([0.0; 3]),
//                 runtime_id: RUNTIME_ID_COUNTER.fetch_add(1, Ordering::SeqCst),
//                 game_mode: GameMode::Creative,
//                 permission_level: PermissionLevel::Member,
//                 command_permission_level: CommandPermissionLevel::Normal,
//                 skin: None,
//                 viewer: ChunkViewer::new(level_manager.clone())
//             }),
//             raknet: RaknetData {
//                 udp_socket: ipv4_socket,
//                 mtu,
//                 guid,
//                 last_update: RwLock::new(Instant::now()),
//                 batch_sequence_number: Default::default(),
//                 sequence_index: Default::default(),
//                 ack_index: Default::default(),
//                 compound_id: Default::default(),
//                 client_batch_number: Default::default(),
//                 compound_collector: Default::default(),
//                 order_channels: Default::default(),
//                 send_queue: Default::default(),
//                 confirmed_packets: Mutex::new(Vec::new()),
//                 compression_enabled: AtomicBool::new(false),
//                 address,
//                 recovery_queue: Default::default(),
//             },
//             broadcast,
//             level: level_manager,
//             replicator,
//             active: CancellationToken::new(),
//             current_tick: AtomicU64::new(0),
//         });

//         // Start processing jobs.
//         // These jobs run in separate tasks, therefore the session has to be cloned.
//         session.clone().start_tick_job();
//         session.clone().start_packet_job(receiver);
//         session
//     }

//     #[inline]
//     pub fn is_initialized(&self) -> bool {
//         self.initialized.load(Ordering::SeqCst)
//     }

//     #[inline]
//     pub fn get_identity_data(&self) -> anyhow::Result<&IdentityData> {
//         self.identity.get().ok_or_else(|| {
//             error!(NotInitialized, "Identity data has not been initialised yet")
//         })
//     }

//     #[inline]
//     pub fn get_user_data(&self) -> anyhow::Result<&UserData> {
//         self.user_data.get().ok_or_else(|| {
//             error!(NotInitialized, "User data has not been initialised yet")
//         })
//     }

//     #[inline]
//     pub fn get_gamemode(&self) -> GameMode {
//         self.player.read().game_mode
//     }

//     #[inline]
//     pub fn get_position(&self) -> Vector<f32, 3> {
//         self.player.read().position.clone()
//     }

//     #[inline]
//     pub fn get_rotation(&self) -> Vector<f32, 3> {
//         self.player.read().rotation.clone()
//     }

//     #[inline]
//     pub fn get_permission_level(&self) -> PermissionLevel {
//         self.player.read().permission_level
//     }

//     #[inline]
//     pub fn get_command_permission_level(&self) -> CommandPermissionLevel {
//         self.player.read().command_permission_level
//     }

//     #[inline]
//     pub fn get_runtime_id(&self) -> u64 {
//         self.player.read().runtime_id
//     }

//     /// Retrieves the identity of the client.
//     #[inline]
//     pub fn get_uuid(&self) -> anyhow::Result<&Uuid> {
//         let identity = self.identity.get().ok_or_else(|| {
//             anyhow!(
//                 "Identity ID data has not been initialised yet"
//             )
//         })?;
//         Ok(&identity.uuid)
//     }

//     /// Retrieves the XUID of the client.
//     #[inline]
//     pub fn get_xuid(&self) -> anyhow::Result<u64> {
//         let identity = self.identity.get().ok_or_else(|| {
//             anyhow!("XUID data has not been initialised yet")
//         })?;
//         Ok(identity.xuid)
//     }

//     /// Retrieves the display name of the client.
//     #[inline]
//     pub fn get_display_name(&self) -> anyhow::Result<&str> {
//         let identity = self.identity.get().ok_or_else(|| {
//             anyhow!(
//                 "Display name data has not been initialised yet"
//             )
//         })?;
//         Ok(&identity.display_name)
//     }

//     #[inline]
//     pub fn get_encryptor(&self) -> anyhow::Result<&Encryptor> {
//         self.encryptor.get().ok_or_else(|| {
//             anyhow!("Encryption has not been initialised yet")
//         })
//     }

//     #[inline]
//     pub fn get_device_os(&self) -> anyhow::Result<DeviceOS> {
//         let data = self.user_data.get().ok_or_else(|| {
//             anyhow!(
//                 "Device OS data has not been initialised yet"
//             )
//         })?;
//         Ok(data.build_platform)
//     }

//     /// Returns the randomly generated GUID given by the client itself.
//     #[inline]
//     pub const fn get_guid(&self) -> u64 {
//         self.raknet.guid
//     }

//     /// Kicks the session from the server, displaying the given menu.
//     pub fn kick<S: AsRef<str>>(&self, message: S) -> anyhow::Result<()> {
//         let disconnect_packet = Disconnect {
//             message: message.as_ref(),
//             hide_message: false,
//         };
//         self.send(disconnect_packet)?;
//         // self.flag_for_close();
//         // FIXME: Client sends disconnect and acknowledgement packet after closing.

//         Ok(())
//     }

//     /// Returns whether the session is currently active.
//     ///
//     /// If this returns false, any remaining associated processes should be stopped as soon as possible.
//     #[inline]
//     pub fn is_active(&self) -> bool {
//         !self.active.is_cancelled()
//     }

//     #[inline]
//     pub async fn cancelled(&self) {
//         self.active.cancelled().await;
//     }
// }
