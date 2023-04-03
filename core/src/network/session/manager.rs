use std::net::SocketAddr;
use std::sync::{Arc, Weak};
use std::time::Duration;

use dashmap::DashMap;
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, mpsc, OnceCell};

use util::{Result, Serialize};
use util::bytes::MutableBuffer;

use crate::network::{
    Disconnect, DISCONNECTED_TIMEOUT,
};
use crate::raknet::{BroadcastPacket, RawPacket};
use crate::{config::SERVER_CONFIG, network::ConnectedPacket};
use crate::level::LevelManager;
use crate::network::Session;

const BROADCAST_CHANNEL_CAPACITY: usize = 16;
const FORWARD_TIMEOUT: Duration = Duration::from_millis(20);
const GARBAGE_COLLECT_INTERVAL: Duration = Duration::from_secs(1);

/// Keeps track of all sessions on the server.
pub struct SessionManager {
    /// Map of all tracked sessions, listed by IP address.
    list: Arc<DashMap<SocketAddr, (mpsc::Sender<MutableBuffer>, Arc<Session>)>>,
    /// The level manager.
    level_manager: OnceCell<Weak<LevelManager>>,
    /// Channel used for packet broadcasting.
    broadcast: broadcast::Sender<BroadcastPacket>,
}

impl SessionManager {
    /// Creates a new session tracker.
    pub fn new() -> Self {
        let list = Arc::new(DashMap::new());
        {
            let list = list.clone();
            tokio::spawn(async move {
                Self::garbage_collector(list).await;
            });
        }

        let (broadcast, _) = broadcast::channel(BROADCAST_CHANNEL_CAPACITY);
        Self {
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
    ) -> anyhow::Result<()> {
        self.level_manager.set(level_manager)?;
        Ok(())
    }

    /// Forwards a packet from the network service to the correct session.
    pub fn forward_packet(&self, packet: RawPacket) {
        // Spawn a new task so that the UDP receiver isn't interrupted
        // if forwarding takes a long amount time.

        let list = self.list.clone();
        tokio::spawn(async move {
            if let Some(session) = list.get(&packet.addr) {
                match session.0.send_timeout(packet.buf, FORWARD_TIMEOUT).await {
                    Ok(_) => (),
                    Err(_) => {
                        // Session incoming queue is full.
                        // If after a 20 ms timeout it is still full, destroy the session,
                        // it probably froze.
                        let xuid = session
                            .1
                            .get_xuid()
                            .map(|x| x.to_string())
                            .unwrap_or_else(|_| "unknown".to_owned());

                        tracing::error!(
                            "It seems like session (with XUID {xuid}) is hanging. Closing it"
                        );

                        // Attempt to send a disconnect packet.
                        let _ = session.1.kick(DISCONNECTED_TIMEOUT);
                        // Then close the session.
                        session.1.on_disconnect();
                    }
                }
            }
        });
    }

    /// Sends the given packet to every session.
    pub fn broadcast<P: ConnectedPacket + Serialize + Clone>(
        &self,
        packet: P,
    ) -> anyhow::Result<()> {
        self.broadcast.send(BroadcastPacket::new(packet, None)?)?;
        Ok(())
    }

    /// Kicks all sessions from the server, displaying the given message.
    /// This function also waits for all sessions to be destroyed.
    pub async fn kick_all<S: AsRef<str>>(&self, message: S) -> anyhow::Result<()> {
        // Ignore result because it can only fail if there are no receivers remaining.
        // In that case this shouldn't do anything anyways.
        let _ = self.broadcast.send(BroadcastPacket::new(
            Disconnect {
                hide_message: false,
                message: message.as_ref(),
            },
            None,
        )?);

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
        list: Arc<DashMap<SocketAddr, (mpsc::Sender<MutableBuffer>, Arc<Session>)>>,
    ) -> ! {
        let mut interval = tokio::time::interval(GARBAGE_COLLECT_INTERVAL);
        loop {
            list.retain(|_, session| -> bool { session.1.is_active() });

            interval.tick().await;
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}