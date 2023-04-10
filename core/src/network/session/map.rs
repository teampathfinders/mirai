use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};
use std::sync::{Arc, Weak};
use std::time::Duration;

use dashmap::DashMap;
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, mpsc, OnceCell};
use tokio_util::sync::CancellationToken;

use util::{Deserialize, Result, Serialize};
use util::bytes::MutableBuffer;
use uuid::Uuid;

use crate::crypto::{UserData, IdentityData};
use crate::network::{CacheStatus, ChunkRadiusReply, ChunkRadiusRequest, Disconnect, DISCONNECTED_TIMEOUT, NETWORK_VERSION, NetworkSettings, Packet, PlayStatus, RequestNetworkSettings, Status, Skin, DeviceOS};
use crate::raknet::{BroadcastPacket, RakNetSession, RawPacket, PacketConfig, Reliability, SendPriority, RakNetSessionBuilder, RakNetMessage};
use crate::{config::SERVER_CONFIG, network::ConnectedPacket};
use crate::instance::UdpController;
use crate::item::ItemRegistry;
use crate::level::LevelManager;

use super::{IncompleteSession, SessionBuilder, Session};

const BROADCAST_CHANNEL_CAPACITY: usize = 16;
const SINGLE_CHANNEL_CAPACITY: usize = 16;
const FORWARD_TIMEOUT: Duration = Duration::from_millis(20);
const GARBAGE_COLLECT_INTERVAL: Duration = Duration::from_secs(1);

/// Methods implement by `session-like` types.
pub trait SessionLike {
    fn send<T>(&self, packet: T) -> anyhow::Result<()>
    where
        T: ConnectedPacket + Serialize;

    fn send_buf<A>(&self, buf: A) -> anyhow::Result<()>
    where
        A: AsRef<[u8]>;

    fn broadcast<T>(&self, packet: T) -> anyhow::Result<()>
    where
        T: ConnectedPacket + Serialize;

    fn broadcast_others<T>(&self, packet: T) -> anyhow::Result<()>
    where
        T: ConnectedPacket + Serialize;

    fn kick<S>(&self, reason: S) -> anyhow::Result<()>
    where
        S: AsRef<str>;
}

/// Contains a session and its packet sender.
#[derive(Debug)]
pub struct SessionRef<T> where T: SessionLike {
    /// Sender that sends packets to the session for processing.
    pub sender: mpsc::Sender<MutableBuffer>,
    /// The actual session.
    pub session: T
}

/// Keeps track of all the sessions connected to the current instance.
#[derive(Debug)]
pub struct SessionMap {
    /// The maximum amount of sessions allowed on this server instance.
    max_sessions: usize,
    /// All sessions that are still in the login sequence.
    incomplete_map: DashMap<SocketAddr, SessionRef<IncompleteSession>>,
    /// All sessions connected to the current instance.
    ///
    /// The sessions are listed by IP and are wrapped in an `Arc`
    /// due to several asynchronous tasks requiring access to them.
    map: DashMap<SocketAddr, SessionRef<Session>>,
    /// Channel used for packet broadcasting.
    ///
    /// Packets sent into this channel will be received by every client connected
    /// to the current instance.
    ///
    /// See [`BroadcastPacket`] for more information.
    broadcast: broadcast::Sender<BroadcastPacket>,
    /// Token that can be cancelled by the instance to make
    /// this controller shut down.
    token: CancellationToken
}

impl SessionMap {
    /// Creates a new session tracker.
    pub fn new(token: CancellationToken, max_sessions: usize) -> Self {
        let incomplete_map = DashMap::new();
        let map = DashMap::new();

        // The receiver end can be created by the sessions.
        let (broadcast, _) = broadcast::channel(BROADCAST_CHANNEL_CAPACITY);

        Self {
            incomplete_map, map, broadcast, token, max_sessions
        }
    }

    /// Adds a new session into the list.
    ///
    /// The method returns `true` if the session was successfully created
    /// and `false` if the server is full.
    pub fn insert(
        &self,
        udp_controller: Arc<UdpController>,
        addr: SocketAddr,
        mtu: u16,
        client_guid: u64
    ) -> bool {
        let mut builder = SessionBuilder::new();
        builder
            .udp_controller(udp_controller)
            .addr(addr)
            .guid(client_guid)
            .mtu(mtu)
            .broadcast(self.broadcast.clone())
            .channel(mpsc::channel(SINGLE_CHANNEL_CAPACITY));

        let session_ref = builder.build();
        if self.count() < self.max_count() {
            self.incomplete_map.insert(addr, session_ref);
            true
        } else {
            false
        }
    }

    pub async fn forward(&self, packet: RawPacket) -> anyhow::Result<()> {
        if let Some(session) = self.map.get(&packet.addr) {
            todo!();
        } else if let Some(session) = self.incomplete_map.get(&packet.addr) {
            let result = session.sender.send_timeout(packet.buf, FORWARD_TIMEOUT).await;
            if result.is_err() {
                anyhow::bail!("Forwarding timed out");
            }
        } else {
            anyhow::bail!("Connected packet received from unconnected client");
        }

        Ok(())
    }

    #[inline]
    pub fn broadcast<T>(&self, packet: T) -> anyhow::Result<()>
    where
        T: ConnectedPacket + Serialize
    {
        self.broadcast.send(BroadcastPacket::new(packet, None)?)?;
        Ok(())
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.incomplete_map.len() + self.map.len()
    }

    #[inline]
    pub fn max_count(&self) -> usize {
        self.max_sessions
    }
}