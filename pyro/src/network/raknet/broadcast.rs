use std::num::NonZeroU64;

use bytes::{Bytes, BytesMut};
use common::{error, Serialize, Result};

use crate::network::{
    packets::{ConnectedPacket, Packet},
    session::Session,
};

#[derive(Debug, Clone)]
pub struct BroadcastPacket {
    /// XUID of the sender of the packet.
    ///
    /// If this is Some, every session that receives the broadcast will check the XUID with its own.
    /// If it matches, the packet will not be sent.
    /// This can be used to broadcast packets to every client other than self.
    pub sender: Option<NonZeroU64>,
    /// Content of the packet.
    ///
    /// This must be an already serialized packet (use the [`Serialize`] trait)
    /// *without* a header.
    pub content: Bytes,
}

impl BroadcastPacket {
    pub fn new<T: ConnectedPacket + Serialize>(
        packet: T,
        sender: Option<NonZeroU64>,
    ) -> Result<Self> {
        let packet = Packet::new(packet);
        
        Ok(Self {
            sender,
            content: packet.serialize()
        })
    }
}

impl Session {
    /// Sends a packet to all initialised sessions including self.
    pub fn broadcast<P: ConnectedPacket + Serialize + Clone>(
        &self,
        pk: P,
    ) -> Result<()> {
        self.broadcast.send(BroadcastPacket::new(pk, None)?)?;
        Ok(())
    }

    /// Sends a packet to all initialised sessions other than self.
    pub fn broadcast_others<P: ConnectedPacket + Serialize + Clone>(
        &self,
        pk: P,
    ) -> Result<()> {
        self.broadcast.send(BroadcastPacket::new(
            pk,
            Some(
                NonZeroU64::new(self.get_xuid()?)
                    .ok_or_else(|| error!(NotInitialized, "XUID was 0"))?,
            ),
        )?)?;

        Ok(())
    }
}
