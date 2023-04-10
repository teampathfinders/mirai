use std::net::SocketAddr;
use std::num::{NonZeroI32, NonZeroU32};
use std::sync::Arc;
use std::sync::atomic::{
    AtomicBool, AtomicU64, Ordering,
};
use std::time::Instant;
use anyhow::anyhow;

use parking_lot::{Mutex, RwLock};
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, mpsc, OnceCell};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use util::{error, Result, Vector, Serialize};
use util::bytes::MutableBuffer;

use crate::network::{DeviceOS, Disconnect, PermissionLevel, ConnectedPacket};
use crate::network::GameMode;
use crate::raknet::{BroadcastPacket, RakNetSession, PacketConfig, SendPriority, Reliability};
use crate::crypto::{Encryptor, IdentityData, UserData};
use crate::item::ItemRegistry;
use crate::level::LevelManager;
use crate::network::Skin;

use super::SessionLike;

static RUNTIME_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct PlayerData {
    /// Whether the player's inventory is currently open.
    pub is_inventory_open: bool,
    /// Position of the player.
    pub position: Vector<f32, 3>,
    /// Rotation of the player.
    /// x and y components are general rotation.
    /// z component is head yaw.
    pub rotation: Vector<f32, 3>,
    /// Render distance of the player in chunks.
    pub render_distance: Option<NonZeroI32>,
    /// Game mode.
    pub game_mode: GameMode,
    /// General permission level.
    pub permission_level: PermissionLevel,
    /// The client's skin.
    pub skin: Option<Skin>,
    /// Runtime ID.
    pub runtime_id: u64,
}

/// Sessions directly correspond to clients connected to the server.
///
/// Anything that has to do with specific clients must be communicated with their associated sessions.
/// The server does not interact with clients directly, everything is done through these sessions.
pub struct Session {
    pub item_registry: Arc<ItemRegistry>,
    /// Identity data such as XUID and display name.
    pub identity: OnceCell<IdentityData>,
    /// Extra user data, such as device OS and language.
    pub user_data: OnceCell<UserData>,
    /// Used to encrypt and decrypt packets.
    pub encryptor: OnceCell<Encryptor>,
    /// Whether the client supports the chunk cache.
    pub cache_support: OnceCell<bool>,
    /// Whether the client has fully been initialised.
    /// This is set to true after receiving the [`SetLocalPlayerAsInitialized`](crate::SetLocalPlayerAsInitialized) packet
    pub initialized: AtomicBool,
    /// Manages entire world.
    pub level_manager: Arc<LevelManager>,
    /// Indicates whether this session is active.
    pub active: CancellationToken,
    /// Current tick of this session, this is increased every [`TICK_INTERVAL`].
    pub current_tick: AtomicU64,
    /// Minecraft-specific data.
    pub player: RwLock<PlayerData>,
    /// Raknet-specific data.
    pub raknet: RakNetSession,
}

impl Session {
    /// Creates a new session.
    pub fn new(

    ) -> Arc<Self> {
        todo!();
    }
}

impl SessionLike for Session {
    fn send<T>(&self, packet: T) -> anyhow::Result<()>
    where
        T: ConnectedPacket + Serialize
    {
        self.raknet.send(packet)
    }

    fn send_buf<A>(&self, buf: A) -> anyhow::Result<()>
    where
        A: AsRef<[u8]>
    {
        self.raknet.send_buf(buf, PacketConfig {
            reliability: Reliability::ReliableOrdered,
            priority: SendPriority::Medium
        })
    }

    fn broadcast<T>(&self, packet: T) -> anyhow::Result<()>
    where
        T: ConnectedPacket + Serialize
    {
        self.raknet.broadcast(packet)
    }

    fn broadcast_others<T>(&self, packet: T) -> anyhow::Result<()>
    where
        T: ConnectedPacket + Serialize
    {
        self.raknet.broadcast_others(packet)
    }
}