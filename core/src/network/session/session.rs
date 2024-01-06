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

const ORDER_CHANNEL_COUNT: usize = 5;

/// The Raknet layer of the user. This handles the entire Raknet protocol for the client.
pub struct RaknetUser {
    // Networking
    pub address: SocketAddr,
    pub socket: Arc<UdpSocket>,

    // Inter-session communication
    pub broadcast: broadcast::Sender<BroadcastPacket>,

    // Raknet data
    pub active: CancellationToken,
    pub mtu: u16,
    /// Keeps track of when the last update was received from the client.
    /// This enables disconnecting users that have lost connection to the server.
    pub last_update: RwLock<Instant>,
    pub tick: AtomicU64,
    /// This client's current batch number. It is increased for every packet batch sent.
    pub batch_number: AtomicU32,

    pub send: SendQueues,

    /// Wrapped in a mutex since reading this will also clear it.
    pub acknowledged: Mutex<Vec<u32>>,
    pub acknowledge_index: AtomicU32,

    pub compound_index: AtomicU16,
    pub compounds: Compounds,

    pub recovery: Recovery,

    pub sequence_index: AtomicU32,
    pub order: [OrderChannel; ORDER_CHANNEL_COUNT],

    pub output: mpsc::Sender<MutableBuffer>
}

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
