use bytes::{Buf, BufMut, BytesMut};

use common::VResult;
use crate::network::packets::GamePacket;
use crate::network::raknet::Header;
use crate::network::traits::Encodable;
use common::{ReadExtensions, WriteExtensions};
use common::vassert;

/// A game packet.
#[derive(Debug)]
pub struct Packet<T: GamePacket> {
    /// Contains the packet ID and subclient IDs.
    header: Header,
    /// Actual packet data.
    internal_packet: T,
}

impl<T: GamePacket> Packet<T> {
    /// Creates a new packet.
    pub const fn new(internal_packet: T) -> Self {
        Self {
            header: Header {
                id: T::ID,
                target_subclient: 0,
                sender_subclient: 0,
            },
            internal_packet,
        }
    }

    /// Sets the subclient IDs.
    pub const fn subclients(mut self, sender: u8, target: u8) -> Self {
        self.header.target_subclient = target;
        self.header.sender_subclient = sender;
        self
    }
}

impl<T: GamePacket + Encodable> Encodable for Packet<T> {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();
        let header = self.header.encode();
        let body = self.internal_packet.encode()?;

        buffer.put_var_u32(header.len() as u32 + body.len() as u32);

        buffer.put(header);
        buffer.put(body);

        Ok(buffer)
    }
}
