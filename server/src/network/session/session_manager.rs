use std::net::SocketAddr;
use std::sync::{Arc, Weak};
use std::time::Duration;

use bytes::Bytes;
use dashmap::DashMap;
use tokio::net::UdpSocket;
use tokio::sync::{OnceCell, broadcast, mpsc};
use tokio_util::sync::CancellationToken;

use crate::instance_manager::InstanceManager;
use crate::level_manager::LevelManager;
use crate::network::packets::login::Disconnect;
use crate::network::raknet::BufPacket;
use crate::network::session::session::Session;
use crate::{config::SERVER_CONFIG, network::packets::GamePacket};
use common::{error, Serialize, VResult};

const BROADCAST_CHANNEL_CAPACITY: usize = 16;
const GARBAGE_COLLECT_INTERVAL: Duration = Duration::from_secs(10);

/// Keeps track of all sessions on the server.
#[derive(Debug)]
pub struct SessionManager {
    /// Whether the server is running.
    /// Once this token is cancelled, the tracker will cancel all the sessions' individual tokens.
    global_token: CancellationToken,
    /// Map of all tracked sessions, listed by IP address.
    list: Arc<DashMap<SocketAddr, (mpsc::Sender<Bytes>, Arc<Session>)>>,
    /// The level manager.
    level_manager: OnceCell<Weak<LevelManager>>,
    /// Channel used for packet broadcasting.
    broadcast: broadcast::Sender<(u64, Bytes)>
}

impl SessionManager {
    /// Creates a new session tracker.
    pub fn new(global_token: CancellationToken) -> Self {
        let list = Arc::new(DashMap::new());
        {
            let list = list.clone();
            tokio::spawn(async move {
                Self::garbage_collector(list).await;
            });
        }

        let (broadcast, _) = broadcast::channel(BROADCAST_CHANNEL_CAPACITY);
        Self {
            global_token,
            list,
            level_manager: OnceCell::new(),
            broadcast,
        }
    }

    /// Creates a new session and adds it to the tracker.
    pub fn add_session(
        self: &Arc<Self>,
        ipv4_socket: Arc<UdpSocket>,
        address: SocketAddr,
        mtu: u16,
        client_guid: u64,
    ) {
        let (sender, receiver) = mpsc::channel(BROADCAST_CHANNEL_CAPACITY);

        let level_manager =
            self.level_manager.get().unwrap().upgrade().unwrap();

        let session = Session::new(
            self.broadcast.clone(),
            receiver,
            level_manager,
            ipv4_socket,
            address,
            mtu,
            client_guid,
        );
        self.list.insert(address, (sender, session));
    }

    #[inline]
    pub fn set_level_manager(
        &self,
        level_manager: Weak<LevelManager>,
    ) -> VResult<()> {
        self.level_manager.set(level_manager)?;
        Ok(())
    }

    /// Forwards a packet from the network service to the correct session.
    pub fn forward_packet(&self, packet: BufPacket) -> VResult<()> {
        self.list
            .get(&packet.addr)
            .map(|r| {
                let session = r.value();
                
            })
            .ok_or_else(|| {
                error!(
                    NotConnected,
                    "Attempted to forward packet to non-existent session"
                )
            })
    }

    pub fn broadcast<P: GamePacket + Serialize + Clone>(&self, pk: P) -> VResult<()> {
        let serialized = pk.serialize()?;
        self.broadcast.send((0, serialized))?;

        Ok(())
    }

    /// Kicks all sessions from the server, displaying the given message.
    pub async fn kick_all<S: AsRef<str>>(&self, message: S) -> VResult<()> {
        let serialized = Disconnect {
            hide_disconnect_screen: false,
            kick_message: message.as_ref()
        }.serialize()?;

        self.broadcast.send((0, serialized))?;
        // Clear to get rid of references
        self.list.clear();

        Ok(())
    }

    /// Returns how many clients are currently connected this tracker.
    #[inline]
    pub fn session_count(&self) -> usize {
        self.list.len()
    }

    /// Returns the maximum amount of sessions this tracker will allow.
    #[inline]
    pub fn max_session_count(&self) -> usize {
        SERVER_CONFIG.read().max_players
    }

    #[inline]
    async fn garbage_collector(
        list: Arc<DashMap<SocketAddr, (mpsc::Sender<Bytes>, Arc<Session>)>>,
    ) -> ! {
        let mut interval = tokio::time::interval(GARBAGE_COLLECT_INTERVAL);
        loop {
            list.retain(|_, session| -> bool { session.1.is_active() });

            interval.tick().await;
        }
    }
}
