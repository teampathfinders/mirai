use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use tokio::net::UdpSocket;
use tokio_util::sync::CancellationToken;

use crate::{config::SERVER_CONFIG, network::packets::GamePacket};
use crate::network::raknet::RawPacket;
use crate::network::session::session::Session;
use common::{error, VResult, Encodable};

const GARBAGE_COLLECT_INTERVAL: Duration = Duration::from_secs(10);

/// Keeps track of all sessions on the server.
#[derive(Debug)]
pub struct SessionTracker {
    /// Whether the server is running.
    /// Once this token is cancelled, the tracker will cancel all the sessions' individual tokens.
    global_token: CancellationToken,
    /// Map of all tracked sessions, listed by IP address.
    session_list: Arc<DashMap<SocketAddr, Arc<Session>>>,
}

impl SessionTracker {
    /// Creates a new session tracker.
    pub fn new(global_token: CancellationToken) -> Self {
        let session_list = Arc::new(DashMap::new());
        {
            let session_list = session_list.clone();
            tokio::spawn(async move {
                Self::garbage_collector(session_list).await;
            });
        }

        Self { global_token, session_list }
    }

    /// Creates a new session and adds it to the tracker.
    pub fn add_session(
        self: &Arc<Self>,
        ipv4_socket: Arc<UdpSocket>,
        address: SocketAddr,
        mtu: u16,
        client_guid: u64,
    ) {
        let session = Session::new(
            Arc::downgrade(self),
            ipv4_socket, address, 
            mtu, client_guid
        );
        self.session_list.insert(address, session);
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

    /// Sends a packet to every *initialized* session on the server.
    pub fn broadcast<P: GamePacket + Encodable>(&self, packet: P) {
        for kv in self.session_list.iter() {
            let session = kv.value();
            // Don't broadcast to uninitialised sessions.
            if session.is_initialized() {
                match session.send(packet.clone()) {
                    Ok(_) => (),
                    Err(e) => {
                        let display_name = session.get_display_name().unwrap_or("unknown session");
                        tracing::error!("Failed to broadcast to {}: {e}", display_name);
                    }
                }
            }
        }
    }

    /// Sends a packet to every *initialized* session on the server except the session with the given XUID.
    pub fn broadcast_except<P: GamePacket + Encodable>(&self, packet: P, xuid: u64) {
        for kv in self.session_list.iter() {
            let session = kv.value();
            let sess_xuid = match session.get_xuid() {
                Ok(x) => x,
                Err(_) => continue 
            };

            // Don't broadcast to uninitialised sessions.
            if session.is_initialized() && xuid != sess_xuid {
                tracing::info!("Sending packet to {}", sess_xuid);

                match session.send(packet.clone()) {
                    Ok(_) => (),
                    Err(e) => {
                        let display_name = session.get_display_name().unwrap_or("unknown session");
                        tracing::error!("Failed to broadcast to {}: {e}", display_name);
                    }
                }
            }
        }
    }

    /// Kicks all sessions from the server, displaying the given message.
    pub async fn kick_all<S: Into<String>>(&self, message: S) {
        // This is separate from broadcast because uninitialised sessions also
        // need to receive this packet.
        // Unlike broadcast, it also flushes all sessions to get rid of them as quickly as possible.

        let message = message.into();
        for x in self.session_list.iter() {
            let session = x.value();
            let _ = session.kick(&message);
            let _ = session.flush_all().await;
        }
    }

    /// Returns how many clients are currently connected this tracker.
    pub fn session_count(&self) -> usize {
        self.session_list.len()
    }

    /// Returns the maximum amount of sessions this tracker will allow.
    pub fn max_session_count(&self) -> usize {
        SERVER_CONFIG.read().max_players
    }

    async fn garbage_collector(
        session_list: Arc<DashMap<SocketAddr, Arc<Session>>>,
    ) -> ! {
        let mut interval = tokio::time::interval(GARBAGE_COLLECT_INTERVAL);
        loop {
            session_list.retain(|_, session| -> bool { session.is_active() });

            interval.tick().await;
        }
    }
}
