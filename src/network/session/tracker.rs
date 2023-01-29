use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use dashmap::DashMap;
use tokio::net::UdpSocket;
use tokio_util::sync::CancellationToken;

use crate::error;
use crate::network::raknet::RawPacket;
use crate::network::session::session::Session;

const GARBAGE_COLLECT_INTERVAL: Duration = Duration::from_secs(10);

/// Keeps track of all sessions on the server.
#[derive(Debug)]
pub struct SessionTracker {
    /// Whether the server is running.
    /// Once this token is cancelled, the tracker will cancel all the sessions' individual tokens.
    global_token: CancellationToken,
    /// Map of all tracked sessions, listed by IP address.
    session_list: Arc<DashMap<SocketAddr, Arc<Session>>>,
    /// Maximum amount of sessions that this tracker will accept.
    max_session_count: usize,
}

impl SessionTracker {
    /// Creates a new session tracker.
    pub fn new(global_token: CancellationToken, max_session_count: usize) -> Self {
        let session_list = Arc::new(DashMap::new());
        {
            let session_list = session_list.clone();
            tokio::spawn(async move {
                Self::garbage_collector(session_list).await;
            });
        }

        Self {
            global_token,
            session_list,
            max_session_count,
        }
    }

    /// Creates a new session and adds it to the tracker.
    pub fn add_session(
        &self,
        ipv4_socket: Arc<UdpSocket>,
        address: SocketAddr,
        mtu: u16,
        client_guid: u64,
    ) {
        let session = Session::new(ipv4_socket, address, mtu, client_guid);
        self.session_list.insert(address, session);
    }

    /// Forwards a packet from the network service to the correct session.
    pub fn forward_packet(&self, packet: RawPacket) -> anyhow::Result<()> {
        self.session_list
            .get(&packet.address)
            .map(|r| {
                let session = r.value();
                session.receive_queue.push(packet.buffer);
            })
            .ok_or(anyhow!(
                "Attempted to forward packet to non-existent session"
            ))
    }

    /// Returns how many clients are currently connected this tracker.
    pub fn session_count(&self) -> usize {
        self.session_list.len()
    }

    /// Returns the maximum amount of sessions this tracker will allow.
    pub const fn max_session_count(&self) -> usize {
        self.max_session_count
    }

    async fn garbage_collector(session_list: Arc<DashMap<SocketAddr, Arc<Session>>>) -> ! {
        let mut interval = tokio::time::interval(GARBAGE_COLLECT_INTERVAL);
        loop {
            session_list.retain(|_, session| -> bool { session.is_active() });

            interval.tick().await;
        }
    }
}
