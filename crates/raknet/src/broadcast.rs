use std::{net::SocketAddr, sync::Arc};

use proto::bedrock::ConnectedPacket;

use util::{RVec, Serialize};

/// A packet that can be broadcast to other sessions.
///
/// Unlike standard raknet, this packet contains an optional sender.
/// As every session listens to a single broadcast channel, this sender field can be used
/// to prevent a session from receiving its own broadcast.
/// In case the session is meant to receive their own packet (such as with the [`Text`](crate::network::TextMessage) packet)
/// this field should be set to `None`.
///
/// Additionally, the actual buffer content is reference counted to allow for cheap cloning.
#[derive(Debug, Clone)]
pub struct BroadcastPacket {
    /// XUID of the sender of the packet.
    ///
    /// If this is Some, every session that receives the broadcast will check the XUID with its own.
    /// If it matches, the packet will not be sent.
    /// This can be used to broadcast raknet to every client other than self.
    pub sender: Option<SocketAddr>,
    /// The ID of the packet.
    pub id: u32,
    /// Content of the packet.
    ///
    /// This must be an already serialized packet (use the [`Serialize`] trait)
    /// *without* a header.
    pub content: Arc<RVec>,
}

impl BroadcastPacket {
    /// Creates a new broadcast packet from the given packet.
    pub fn new<T: ConnectedPacket + Serialize>(
        packet: T,
        sender: Option<SocketAddr>,
        // algorithm: Option<CompressionAlgorithm>
    ) -> anyhow::Result<Self> {
        Ok(Self {
            sender,
            id: T::ID,
            content: Arc::from(packet.serialize()?),
        })
    }
}
