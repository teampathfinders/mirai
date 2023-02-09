use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use tokio::net::UdpSocket;
use tokio_util::sync::CancellationToken;

use vex_common::{error, SERVER_CONFIG};
use vex_common::VResult;

use crate::{ConnectCallback, MessageCallback};
use crate::raw::RawPacket;
use crate::session::Session;

const GARBAGE_COLLECT_INTERVAL: Duration = Duration::from_secs(10);

type SessionMap = Arc<DashMap<SocketAddr, Arc<Session>>>;

/// Keeps track of all sessions on the server.
#[derive(Debug)]
pub struct SessionTracker {
    /// Whether the server is running.
    /// Once this token is cancelled, the tracker will cancel all the sessions' individual tokens.
    token: CancellationToken,
    /// Map of all tracked sessions, listed by IP address.
    session_list: SessionMap,
    callback: ConnectCallback,
}

impl SessionTracker {
    /// Creates a new session tracker.
    pub fn new(token: CancellationToken, callback: ConnectCallback) -> Self {
        let session_list = Arc::new(DashMap::new());
        {
            let session_list = session_list.clone();
            tokio::spawn(async move {
                Self::garbage_collector(session_list).await;
            });
        }

        Self {
            token,
            session_list,
            callback,
        }
    }

    /// Creates a new session and adds it to the tracker.
    pub fn add_session(
        &self,
        ipv4_socket: Arc<UdpSocket>,
        address: SocketAddr,
        mtu: u16,
        client_guid: u64,
    ) -> VResult<()> {
        let session = Session::new(ipv4_socket, address, mtu, client_guid);
        let message_callback = (self.callback.0)(session.clone())?;

        session.callback.set(MessageCallback(message_callback))?;

        self.session_list.insert(address, session);
        Ok(())
    }

    /// Forwards a packet from the network service to the correct session.
    pub fn forward_packet(&self, packet: RawPacket) -> VResult<()> {
        self.session_list
            .get(&packet.address)
            .map(|r| {
                let session = r.value();
                session.receive_queue.push(packet.buffer);
            })
            .ok_or_else(|| {
                error!(
                    NotConnected,
                    "Attempted to forward packet to non-existent session"
                )
            })
    }

    /// Returns how many clients are currently connected this tracker.
    pub fn session_count(&self) -> usize {
        self.session_list.len()
    }

    /// Returns the maximum amount of sessions this tracker will allow.
    pub fn max_session_count(&self) -> usize {
        SERVER_CONFIG.read().max_players
    }

    async fn garbage_collector(session_map: SessionMap) -> ! {
        let mut interval = tokio::time::interval(GARBAGE_COLLECT_INTERVAL);
        loop {
            session_map.retain(|_, session| -> bool { session.is_active() });

            interval.tick().await;
        }
    }
}
