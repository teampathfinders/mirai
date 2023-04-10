use std::net::SocketAddr;
use std::sync::{Arc, Weak};
use std::time::Duration;

use dashmap::DashMap;
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, mpsc, OnceCell};
use tokio_util::sync::CancellationToken;

use util::{Result, Serialize};
use util::bytes::MutableBuffer;

use crate::network::{
    Disconnect, DISCONNECTED_TIMEOUT,
};
use crate::raknet::{BroadcastPacket, RawPacket};
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
    sender: Option<mpsc::Sender<RawPacket>>,
    receiver: Option<mpsc::Receiver<RawPacket>>,
    broadcast: Option<broadcast::Sender<BroadcastPacket>>,
    guid: u64
}

impl SessionBuilder {
    /// Creates a new `SessionBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Configures the Raknet GUID of the session.
    pub fn guid(&mut self, guid: u64) -> &mut Self {
        self.guid = guid;
        self
    }

    /// Configures the channel used for packet sending/receiving.
    pub fn channel(
        &mut self,
        (sender, receiver): (mpsc::Sender<RawPacket>, mpsc::Receiver<RawPacket>)
    ) -> &mut Self {
        self.receiver = Some(receiver);
        self.sender = Some(sender);
        self
    }

    /// Configures the channel used for packet broadcasting.
    pub fn broadcast(&mut self, broadcast: broadcast::Sender<BroadcastPacket>) -> &mut Self {
        self.broadcast = Some(broadcast);
        self
    }

    /// Builds a [`SessionRef`] and consumes this builder.
    ///
    /// # Panics
    ///
    /// This method panics if several required options were not set.
    pub fn build(mut self) -> SessionRef<IncompleteSession> {
        let sender = self.sender.take().unwrap();
        let session = IncompleteSession::from(self);

        SessionRef {
            sender, session
        }
    }
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
    broadcast: broadcast::Sender<BroadcastPacket>,
    receiver: mpsc::Receiver<RawPacket>,

    guid: u64
}

impl IncompleteSession {
    
}

impl From<SessionBuilder> for IncompleteSession {
    fn from(builder: SessionBuilder) -> Self {
        Self {
            broadcast: builder.broadcast.unwrap(),
            receiver: builder.receiver.unwrap(),
            guid: builder.guid
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
    pub fn new(token: CancellationToken) -> Self {
        let incomplete_map = DashMap::new();
        let map = DashMap::new();

        // The receiver end can be created by the sessions.
        let (broadcast, _) = broadcast::channel(BROADCAST_CHANNEL_CAPACITY);

        Self {
            incomplete_map, map, broadcast, token
        }
    }

    /// Adds a new session into the list.
    pub fn insert(
        &self,
        udp_controller: Arc<UdpController>,
        addr: SocketAddr,
        mtu: u16,
        client_guid: u64
    ) {
        let session_ref = SessionBuilder::new()
            .guid(client_guid)
            .broadcast(self.broadcast.clone())
            .channel(mpsc::channel(SINGLE_CHANNEL_CAPACITY))
            .build();

        self.map.insert(addr, session_ref);
    }


}