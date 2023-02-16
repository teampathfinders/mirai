use std::net::SocketAddr;
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering, AtomicU16};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

use bytes::BytesMut;
use parking_lot::{Mutex, RwLock, RwLockReadGuard};
use tokio::net::UdpSocket;
use tokio::sync::OnceCell;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::crypto::{Encryptor, IdentityData, UserData};
use crate::instance_manager::{InstanceManager};
use crate::level_manager::LevelManager;
use crate::network::session::compound_collector::CompoundCollector;
use crate::network::session::order_channel::OrderChannel;
use crate::network::session::recovery_queue::RecoveryQueue;
use crate::network::session::send_queue::SendQueue;
use common::{AsyncDeque, Encodable, bail};
use common::{error, VResult};
use crate::network::packets::login::{DeviceOS, Disconnect};
use crate::network::packets::{GamePacket, MessageType, PlayerListRemove, TextMessage};
use crate::network::Skin;

use super::SessionManager;

/// Tick interval of the internal session ticker.
const INTERNAL_TICK_INTERVAL: Duration = Duration::from_millis(1000 / 20);
/// Tick interval for session packet processing.
const TICK_INTERVAL: Duration = Duration::from_millis(1000 / 20);
/// Inactivity timeout.
///
/// Any sessions that do not respond within this specified timeout will be disconnect from the server.
/// Timeouts can happen if a client's game crashed for example.
/// They will stop responding to the server, but will not explicitly send a disconnect request.
/// Hence, they have to be disconnected manually after the timeout passes.
const SESSION_TIMEOUT: Duration = Duration::from_secs(5);

const ORDER_CHANNEL_COUNT: usize = 5;

static RUNTIME_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Sessions directly correspond to clients connected to the server.
///
/// Anything that has to do with specific clients must be communicated with their associated sessions.
/// The server does not interact with clients directly, everything is done through these sessions.
#[derive(Debug)]
pub struct Session {
    /// Identity data such as XUID and display name.
    pub identity: OnceCell<IdentityData>,
    /// Extra user data, such as device OS and language.
    pub user_data: OnceCell<UserData>,
    /// The client's skin.
    pub skin: RwLock<Option<Skin>>,
    /// Used to encrypt and decrypt packets.
    pub encryptor: OnceCell<Encryptor>,
    /// Whether the client supports the chunk cache.
    pub cache_support: OnceCell<bool>,
    /// Whether the client has fully been initialised.
    /// This is set to true after receiving the [`SetLocalPlayerAsInitialized`](crate::network::packets::SetLocalPlayerAsInitialized) packet
    pub initialized: AtomicBool,
    /// Runtime ID.
    pub runtime_id: u64,
    /// Session list, used to access broadcasting.
    /// This is a weak pointer because of the cyclic reference.
    /// In principle, the tracker should always exist if there are sessions remaining.
    pub session_manager: Arc<SessionManager>,
    pub level_manager: Arc<LevelManager>,

    /// Current tick of this session, this is increased by one every time the session
    /// processes packets.
    pub current_tick: AtomicU64,
    /// IPv4 socket of the server.
    pub ipv4_socket: Arc<UdpSocket>,
    /// IP address of this session.
    pub address: SocketAddr,
    /// Maximum packet size
    pub mtu: u16,
    /// Client-provided GUID.
    /// These IDs are randomly generated by Minecraft for each connection and are unreliable.
    /// They should not be used as unique identifiers, use the XUID instead.
    pub guid: u64,
    /// Timestamp of when the last packet was received from this client.
    pub last_update: RwLock<Instant>,
    /// Indicates whether this session is active.
    pub active: CancellationToken,
    /// Batch number last assigned by the server.
    pub batch_number: AtomicU32,
    /// Sequence index last assigned by the server.
    pub sequence_index: AtomicU32,
    /// Acknowledgment index last used by the server.
    pub acknowledgment_index: AtomicU32,
    /// Compound ID last used by the server.
    pub compound_id: AtomicU16,
    /// Latest sequence index that was received.
    /// Sequenced packets with sequence numbers less than this one will be discarded.
    pub client_batch_number: AtomicU32,
    /// Collects fragmented packets.
    pub compound_collector: CompoundCollector,
    /// Channels used to order packets.
    pub order_channels: [OrderChannel; ORDER_CHANNEL_COUNT],
    /// Keeps track of all packets that are waiting to be sent.
    pub send_queue: SendQueue,
    /// Packets that are ready to be acknowledged.
    pub confirmed_packets: Mutex<Vec<u32>>,
    /// Whether compression has been configured for this session.
    pub compression_enabled: AtomicBool,
    /// Keeps track of all unprocessed received packets.
    pub receive_queue: AsyncDeque<BytesMut>,
    /// Queue that stores packets in case they need to be recovered due to packet loss.
    pub recovery_queue: RecoveryQueue,
}

impl Session {
    /// Creates a new session.
    pub fn new(
        session_manager: Arc<SessionManager>,
        level_manager: Arc<LevelManager>,
        ipv4_socket: Arc<UdpSocket>,
        address: SocketAddr,
        mtu: u16,
        guid: u64,
    ) -> Arc<Self> {
        let session = Arc::new(Self {
            identity: OnceCell::new(),
            user_data: OnceCell::new(),
            skin: RwLock::new(None),
            encryptor: OnceCell::new(),
            cache_support: OnceCell::new(),
            initialized: AtomicBool::new(false),
            runtime_id: RUNTIME_ID_COUNTER.fetch_add(1, Ordering::SeqCst),
            session_manager,
            level_manager,

            current_tick: AtomicU64::new(0),
            ipv4_socket,
            mtu,
            guid,
            last_update: RwLock::new(Instant::now()),
            active: CancellationToken::new(),
            batch_number: Default::default(),
            sequence_index: Default::default(),
            acknowledgment_index: Default::default(),
            compound_id: Default::default(),
            client_batch_number: Default::default(),
            compound_collector: Default::default(),
            order_channels: Default::default(),
            send_queue: Default::default(),
            confirmed_packets: Mutex::new(Vec::new()),
            compression_enabled: AtomicBool::new(false),
            receive_queue: AsyncDeque::new(5),
            address,
            recovery_queue: Default::default()
        });

        // Start ticker
        {
            let session = session.clone();
            tokio::spawn(async move {
                let mut interval =
                    tokio::time::interval(INTERNAL_TICK_INTERVAL);
                while !session.active.is_cancelled() {
                    match session.tick().await {
                        Ok(_) => (),
                        Err(e) => tracing::error!("{e}"),
                    }
                    interval.tick().await;
                }

                // Flush last acknowledgements before closing
                match session.flush_acknowledgements().await {
                    Ok(_) => (),
                    Err(e) => {
                        tracing::error!(
                            "Failed to flush last acknowledgements before session close"
                        );
                    }
                }

                // Flush last packets before closing
                match session.flush().await {
                    Ok(_) => (),
                    Err(e) => {
                        tracing::error!(
                            "Failed to flush last packets before session close"
                        );
                    }
                }
            });
        }

        // Start processor
        {
            let session = session.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(TICK_INTERVAL);
                while !session.active.is_cancelled() {
                    match session.handle_raw_packet().await {
                        Ok(_) => (),
                        Err(e) => tracing::error!("{e}"),
                    }
                    interval.tick().await;
                }
            });
        }

        session
    }

    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::SeqCst)
    }

    pub fn get_identity_data(&self) -> VResult<&IdentityData> {
        self.identity.get().ok_or_else(|| {
            error!(NotInitialized, "Identity data has not been initialised yet")
        })
    }

    pub  fn get_user_data(&self) -> VResult<&UserData> {
        self.user_data.get().ok_or_else(|| {
            error!(NotInitialized, "User data has not been initialised yet")
        })
    }

    /// Retrieves the identity of the client.
    pub fn get_uuid(&self) -> VResult<&Uuid> {
        let identity = self.identity.get().ok_or_else(|| {
            error!(NotInitialized, "Identity ID data has not been initialised yet")
        })?;
        Ok(&identity.uuid)
    }

    /// Retrieves the XUID of the client.
    pub fn get_xuid(&self) -> VResult<u64> {
        let identity = self.identity.get().ok_or_else(|| {
            error!(NotInitialized, "XUID data has not been initialised yet")
        })?;
        Ok(identity.xuid)
    }

    /// Retrieves the display name of the client.
    pub fn get_display_name(&self) -> VResult<&str> {
        let identity = self.identity.get().ok_or_else(|| {
            error!(NotInitialized, "Display name data has not been initialised yet")
        })?;
        Ok(&identity.display_name)
    }

    pub fn get_encryptor(&self) -> VResult<&Encryptor> {
        self.encryptor.get().ok_or_else(|| {
            error!(NotInitialized, "Encryption has not been initialised yet")
        })
    }

    pub fn get_device_os(&self) -> VResult<DeviceOS> {
        let data = self.user_data.get().ok_or_else(|| {
            error!(NotInitialized, "Device OS data has not been initialised yet")
        })?;
        Ok(data.build_platform)
    }

    /// Returns the randomly generated GUID given by the client itself.
    pub const fn get_guid(&self) -> u64 {
        self.guid
    }

    /// Signals to the session that it needs to close.
    pub fn flag_for_close(&self) {
        self.initialized.store(false, Ordering::SeqCst);

        if let Ok(display_name) = self.get_display_name() {
            if let Ok(uuid) = self.get_uuid() {
                tracing::info!("{display_name} has disconnected");
                let _ = self.broadcast_others(TextMessage {
                    message: format!("§e{display_name} has left the server."),
                    message_type: MessageType::System,
                    needs_translation: false,
                    parameters: vec![],
                    platform_chat_id: "".to_owned(),
                    source_name: "".to_owned(),
                    xuid: "".to_owned()
                });
    
                let _ = self.broadcast_others(PlayerListRemove {
                    entries: &[*uuid]
                });
            }
        }
        self.active.cancel();
    }

    /// Sends a packet to all initialised sessions including self.
    pub fn broadcast<P: GamePacket + Encodable>(&self, packet: P) -> VResult<()> {
        // Upgrade weak pointer to use it.
        // let tracker = self.tracker
        //     .upgrade()
        //     .ok_or_else(|| error!(NotInitialized, "Session attempted to use tracker that does not exist anymore. This is definitely a bug"))?;

        self.session_manager.broadcast(packet);
        Ok(())
    }

    /// Sends a packet to all initialised sessions other than self.
    pub fn broadcast_others<P: GamePacket + Encodable>(&self, packet: P) -> VResult<()> {
        self.session_manager.broadcast_except(packet, self.get_xuid()?);
        Ok(())
    }

    /// Kicks the session from the server, displaying the given menu.
    pub fn kick<S: AsRef<str>>(&self, message: S) -> VResult<()> {
        let disconnect_packet = Disconnect {
            kick_message: message.as_ref(),
            hide_disconnect_screen: false,
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

    /// Performs tasks not related to packet processing
    async fn tick(self: &Arc<Self>) -> VResult<()> {
        let current_tick = self.current_tick.fetch_add(1, Ordering::SeqCst);

        // Session has timed out
        if Instant::now().duration_since(*self.last_update.read())
            > SESSION_TIMEOUT
        {
            self.flag_for_close();
        }

        self.flush().await?;
        Ok(())
    }

    /// Called by the [`SessionTracker`](super::tracker::SessionTracker) to forward packets from the network service to
    /// the session corresponding to the client.
    pub fn on_packet_received(self: &Arc<Self>, buffer: BytesMut) {
        self.receive_queue.push(buffer);
    }
}