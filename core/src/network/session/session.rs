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

use util::{error, Result, Vector};
use util::bytes::MutableBuffer;

use crate::network::{DeviceOS, Disconnect, PermissionLevel};
use crate::network::GameMode;
use crate::raknet::{BroadcastPacket, RakNetSession};
use crate::crypto::{Encryptor, IdentityData, UserData};
use crate::item::ItemRegistry;
use crate::level::LevelManager;
use crate::network::Skin;

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
        broadcast: broadcast::Sender<BroadcastPacket>,
        receiver: mpsc::Receiver<MutableBuffer>,
        item_registry: Arc<ItemRegistry>,
        level_manager: Arc<LevelManager>,
        ipv4_socket: Arc<UdpSocket>,
        address: SocketAddr,
        mtu: u16,
        guid: u64,
    ) -> Arc<Self> {
        let session = Arc::new(Self {
            item_registry,
            identity: OnceCell::new(),
            user_data: OnceCell::new(),
            encryptor: OnceCell::new(),
            cache_support: OnceCell::new(),
            initialized: AtomicBool::new(false),
            player: RwLock::new(PlayerData {
                is_inventory_open: false,
                position: Vector::from([23.0, 23.0, 2.0]),
                rotation: Vector::from([0.0; 3]),
                render_distance: None,
                runtime_id: RUNTIME_ID_COUNTER.fetch_add(1, Ordering::SeqCst),
                game_mode: GameMode::Survival,
                permission_level: PermissionLevel::Member,
                skin: None,
            }),
            raknet: RakNetSession {
                udp_controller: ipv4_socket,
                mtu,
                guid,
                last_update: RwLock::new(Instant::now()),
                batch_number: Default::default(),
                sequence_index: Default::default(),
                ack_index: Default::default(),
                compound_id: Default::default(),
                client_batch_number: Default::default(),
                compound_collector: Default::default(),
                order_channels: Default::default(),
                send_queue: Default::default(),
                confirmed_packets: Mutex::new(Vec::new()),
                compression_enabled: AtomicBool::new(false),
                addr: address,
                recovery_queue: Default::default(),
            },
            broadcast,
            level_manager,
            active: CancellationToken::new(),
            current_tick: AtomicU64::new(0),
        });

        // Start processing jobs.
        // These jobs run in separate tasks, therefore the session has to be cloned.
        session.clone().start_tick_job();
        session.clone().start_packet_job(receiver);
        session
    }

    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::SeqCst)
    }

    #[inline]
    pub fn get_identity_data(&self) -> anyhow::Result<&IdentityData> {
        self.identity.get().ok_or_else(|| {
            error!(NotInitialized, "Identity data has not been initialised yet")
        })
    }

    #[inline]
    pub fn get_user_data(&self) -> anyhow::Result<&UserData> {
        self.user_data.get().ok_or_else(|| {
            error!(NotInitialized, "User data has not been initialised yet")
        })
    }

    #[inline]
    pub fn get_game_mode(&self) -> GameMode {
        self.player.read().game_mode
    }

    #[inline]
    pub fn get_position(&self) -> Vector<f32, 3> {
        self.player.read().position.clone()
    }

    #[inline]
    pub fn get_rotation(&self) -> Vector<f32, 3> {
        self.player.read().rotation.clone()
    }

    #[inline]
    pub fn get_permission_level(&self) -> PermissionLevel {
        self.player.read().permission_level
    }

    /// Retrieves the identity of the client.
    #[inline]
    pub fn get_uuid(&self) -> anyhow::Result<&Uuid> {
        let identity = self.identity.get().ok_or_else(|| {
            anyhow!(
                "Identity ID data has not been initialised yet"
            )
        })?;
        Ok(&identity.uuid)
    }

    /// Retrieves the XUID of the client.
    #[inline]
    pub fn get_xuid(&self) -> anyhow::Result<u64> {
        let identity = self.identity.get().ok_or_else(|| {
            anyhow!("XUID data has not been initialised yet")
        })?;
        Ok(identity.xuid)
    }

    /// Retrieves the display name of the client.
    #[inline]
    pub fn get_display_name(&self) -> anyhow::Result<&str> {
        let identity = self.identity.get().ok_or_else(|| {
            anyhow!(
                "Display name data has not been initialised yet"
            )
        })?;
        Ok(&identity.display_name)
    }

    #[inline]
    pub fn get_encryptor(&self) -> anyhow::Result<&Encryptor> {
        self.encryptor.get().ok_or_else(|| {
            anyhow!("Encryption has not been initialised yet")
        })
    }

    #[inline]
    pub fn get_device_os(&self) -> anyhow::Result<DeviceOS> {
        let data = self.user_data.get().ok_or_else(|| {
            anyhow!(
                "Device OS data has not been initialised yet"
            )
        })?;
        Ok(data.build_platform)
    }

    /// Returns the randomly generated GUID given by the client itself.
    #[inline]
    pub const fn get_guid(&self) -> u64 {
        self.raknet.guid
    }

    /// Kicks the session from the server, displaying the given menu.
    pub fn kick<S: AsRef<str>>(&self, message: S) -> anyhow::Result<()> {
        let disconnect_packet = Disconnect {
            message: message.as_ref(),
            hide_message: false,
        };
        self.send(disconnect_packet)?;
        // self.flag_for_close();
        // FIXME: Client sends disconnect and acknowledgement packet after closing.

        Ok(())
    }

    /// Returns whether the session is currently active.
    ///
    /// If this returns false, any remaining associated processes should be stopped as soon as possible.
    #[inline]
    pub fn is_active(&self) -> bool {
        !self.active.is_cancelled()
    }

    #[inline]
    pub async fn cancelled(&self) {
        self.active.cancelled().await;
    }
}
