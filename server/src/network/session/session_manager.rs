use std::net::SocketAddr;
use std::sync::{Arc, Weak};
use std::time::Duration;

use bytes::Bytes;
use dashmap::DashMap;
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, mpsc, OnceCell};
use tokio_util::sync::CancellationToken;

use crate::instance_manager::InstanceManager;
use crate::level_manager::LevelManager;
use crate::network::packets::login::Disconnect;
use crate::network::packets::{BroadcastPacket, Packet};
use crate::network::raknet::BufPacket;
use crate::network::session::session::Session;
use crate::{config::SERVER_CONFIG, network::packets::ConnectedPacket};
use common::{bail, error, Serialize, VResult};

const BROADCAST_CHANNEL_CAPACITY: usize = 16;
const FORWARD_TIMEOUT: Duration = Duration::from_millis(20);
const GARBAGE_COLLECT_INTERVAL: Duration = Duration::from_secs(1);

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
    broadcast: broadcast::Sender<BroadcastPacket>,
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

        // Lightweight task that removes the session from the list when it is no longer active.
        // This prevents cyclic references.
        {
            let list = self.list.clone();
            let session = session.clone();

            tokio::spawn(async move {
                session.cancelled().await;
                list.remove(&session.raknet.address);
            });
        }

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
    pub fn forward_packet(&self, pk: BufPacket) {
        // Spawn a new task so that the UDP receiver isn't interrupted
        // if forwarding takes a long amount time.

        let list = self.list.clone();
        tokio::spawn(async move {
            if let Some(session) = list.get(&pk.addr) {
                match session.0.send_timeout(pk.buf, FORWARD_TIMEOUT).await {
                    Ok(_) => (),
                    Err(_) => {
                        // Session incoming queue is full.
                        // If after a 20 ms timeout it is still full, destroy the session,
                        // it probably froze.
                        tracing::error!(
                            "Session queue is full, it seems like the session is hanging. Closing it"
                        );
                        session.1.flag_for_close();
                    }
                }
            } else {
                tracing::error!(
                    "Received online packet for unconnected client"
                );
            }
        });
    }

    /// Sends the given packet to every session.
    pub fn broadcast<P: ConnectedPacket + Serialize + Clone>(
        &self,
        pk: P,
    ) -> VResult<()> {
        self.broadcast.send(BroadcastPacket::new(pk, None)?)?;
        Ok(())
    }

    /// Kicks all sessions from the server, displaying the given message.
    /// This function also waits for all sessions to be destroyed.
    pub async fn kick_all<S: AsRef<str>>(&self, message: S) -> VResult<()> {
        self.broadcast.send(BroadcastPacket::new(
            Disconnect {
                hide_disconnect_screen: false,
                kick_message: message.as_ref(),
            },
            None,
        )?)?;

        for session in self.list.iter() {
            session.value().1.cancelled().await;
        }

        // Clear to get rid of references, so that the sessions are dropped once they're ready.
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
