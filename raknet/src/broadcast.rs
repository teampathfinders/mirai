use std::{net::SocketAddr, sync::Arc};

use proto::bedrock::{ConnectedPacket, Packet};

use util::Serialize;

use crate::RaknetUser;

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
    /// Content of the packet.
    ///
    /// This must be an already serialized packet (use the [`Serialize`] trait)
    /// *without* a header.
    pub content: Arc<Vec<u8>>,
}

impl BroadcastPacket {
    /// Creates a new broadcast packet from the given packet.
    pub fn new<T: ConnectedPacket + Serialize>(
        packet: T,
        sender: Option<SocketAddr>,
    ) -> anyhow::Result<Self> {
        let packet = Packet::new(packet);

        Ok(Self {
            sender,
            content: Arc::from(packet.serialize()?),
        })
    }
}

impl RaknetUser {
    /// Sends a packet to all initialised sessions including self.
    pub fn broadcast<P: ConnectedPacket + Serialize + Clone>(
        &self,
        packet: P,
    ) -> anyhow::Result<()> {
        self.broadcast.send(BroadcastPacket::new(packet, None)?)?;
        Ok(())
    }

    /// Sends a packet to all initialised sessions other than self.
    pub fn broadcast_others<P: ConnectedPacket + Serialize + Clone>(
        &self,
        packet: P,
    ) -> anyhow::Result<()> {
        self.broadcast.send(BroadcastPacket::new(
            packet,
            Some(
                self.address
            ),
        )?)?;

        Ok(())
    }
}
