use std::net::SocketAddr;
use std::sync::{Arc, Weak};
use std::time::Duration;

use dashmap::DashMap;
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, mpsc, OnceCell};
use tokio_util::sync::CancellationToken;

use util::{Deserialize, Result, Serialize};
use util::bytes::MutableBuffer;

use crate::network::{CacheStatus, ChunkRadiusReply, ChunkRadiusRequest, Disconnect, DISCONNECTED_TIMEOUT, NETWORK_VERSION, NetworkSettings, Packet, PlayStatus, RequestNetworkSettings, Status};
use crate::raknet::{BroadcastPacket, RakNetSession, RawPacket, PacketConfig, Reliability, SendPriority, RakNetSessionBuilder};
use crate::{config::SERVER_CONFIG, network::ConnectedPacket};
use crate::instance::UdpController;
use crate::item::ItemRegistry;
use crate::level::LevelManager;
// use crate::network::Session;

const BROADCAST_CHANNEL_CAPACITY: usize = 16;
const SINGLE_CHANNEL_CAPACITY: usize = 16;
const FORWARD_TIMEOUT: Duration = Duration::from_millis(20);
const GARBAGE_COLLECT_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Default)]
pub struct SessionBuilder {
    addr: Option<SocketAddr>,
    udp: Option<Arc<UdpController>>,
    sender: Option<mpsc::Sender<RawPacket>>,
    receiver: Option<mpsc::Receiver<RawPacket>>,
    broadcast: Option<broadcast::Sender<BroadcastPacket>>,
    guid: u64
}

impl SessionBuilder {
    /// Creates a new `SessionBuilder`.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Configures the Raknet GUID of the session.
    #[inline]
    pub fn guid(&mut self, guid: u64) -> &mut Self {
        self.guid = guid;
        self
    }

    #[inline]
    pub fn udp(&mut self, controller: Arc<UdpController>) -> &mut Self {
        self.udp = Some(controller);
        self
    }

    #[inline]
    pub fn addr(&mut self, addr: SocketAddr) -> &mut Self {
        self.addr = Some(addr);
        self
    }

    /// Configures the channel used for packet sending/receiving.
    #[inline]
    pub fn channel(
        &mut self,
        (sender, receiver): (mpsc::Sender<RawPacket>, mpsc::Receiver<RawPacket>)
    ) -> &mut Self {
        self.receiver = Some(receiver);
        self.sender = Some(sender);
        self
    }

    /// Configures the channel used for packet broadcasting.
    #[inline]
    pub fn broadcast(&mut self, broadcast: broadcast::Sender<BroadcastPacket>) -> &mut Self {
        self.broadcast = Some(broadcast);
        self
    }

    /// Builds a [`SessionRef`] and consumes this builder.
    ///
    /// # Panics
    ///
    /// This method panics if several required options were not set.
    #[inline]
    pub fn build(mut self) -> SessionRef<IncompleteSession> {
        let sender = self.sender.take().unwrap();
        let session = IncompleteSession::from(self);

        SessionRef {
            sender, session
        }
    }
}

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
}

pub struct Session {
    broadcast: broadcast::Sender<BroadcastPacket>,
    receiver: mpsc::Receiver<RawPacket>
}

impl From<SessionBuilder> for Session {
    fn from(builder: SessionBuilder) -> Self {
        Self {
            broadcast: builder.broadcast.unwrap(),
            receiver: builder.receiver.unwrap()
        }
    }
}

pub struct IncompleteSession {
    expected: u32,
    cache_support: bool,
    render_distance: i32,
    guid: u64,
    compression: bool,
    raknet: RakNetSession
}

impl IncompleteSession {
    pub fn on_cache_status(&mut self, packet: MutableBuffer) -> anyhow::Result<()> {
        let status = CacheStatus::deserialize(packet.as_ref())?;
        self.cache_support = status.support;

        Ok(())
    }

    pub fn on_radius_request(&mut self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = ChunkRadiusRequest::deserialize(packet.as_ref())?;
        let radius = std::cmp::min(SERVER_CONFIG.read().allowed_render_distance, request.radius);

        if request.radius <= 0 {
            anyhow::bail!("Render distance must be greater than 0");
        }

        self.send(ChunkRadiusReply {
            radius
        })?;

        self.render_distance = radius;
        Ok(())
    }

    pub fn on_settings_request(&mut self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = RequestNetworkSettings::deserialize(packet.as_ref())?;
        if request.protocol_version > NETWORK_VERSION {
            self.send(PlayStatus {
                status: Status::FailedServer
            })?;

            anyhow::bail!(format!(
                "Client is using a newer protocol version: {} vs. {NETWORK_VERSION}",
                request.protocol_version
            ));
        } else if request.protocol_version < NETWORK_VERSION {
            self.send(PlayStatus {
                status: Status::FailedClient
            })?;

            anyhow::bail!(format!(
                "Client is using an older protocol version: {} vs. {NETWORK_VERSION}",
                request.protocol_version
            ));
        }

        let response = {
            let lock = SERVER_CONFIG.read();

            NetworkSettings {
                compression_algorithm: lock.compression_algorithm,
                compression_threshold: lock.compression_threshold,
                client_throttle: lock.client_throttle
            }
        };

        self.compression = true;
        self.send(response)
    }
}

impl SessionLike for IncompleteSession {
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
        T: ConnectedPacket + Serialize,
    {
        self.raknet.broadcast_others(packet)
    }
}

impl From<SessionBuilder> for IncompleteSession {
    fn from(builder: SessionBuilder) -> Self {
        let raknet = RakNetSessionBuilder::new()
            .udp(builder.udp.unwrap())
            .addr(builder.addr.unwrap())
            .broadcast(builder.broadcast.unwrap())
            .receiver(builder.receiver.unwrap())
            .guid(builder.guid)
            .build();

        Self {
            expected: 0,
            cache_support: false,
            render_distance: 0,
            guid: builder.guid,
            compression: false,
            raknet
        }
    }
}

/// Contains a session and its packet sender.
pub struct SessionRef<T> {
    /// Sender that sends packets to the session for processing.
    pub sender: mpsc::Sender<RawPacket>,
    /// The actual session.
    pub session: T
}

/// Keeps track of all the sessions connected to the current instance.
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
        let session_ref = SessionBuilder::new()
            .guid(client_guid)
            .broadcast(self.broadcast.clone())
            .channel(mpsc::channel(SINGLE_CHANNEL_CAPACITY))
            .build();

        if self.count() < self.max_count() {
            self.incomplete_map.insert(addr, session_ref);
            true
        } else {
            false
        }
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