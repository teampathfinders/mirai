use std::num::NonZeroU64;

use bytes::Bytes;
use bytes::{Buf, BufMut, BytesMut};

use crate::network::header::Header;
use crate::network::packets::ConnectedPacket;
use common::nvassert;
use common::Serialize;
use common::VResult;
use common::{ReadExtensions, WriteExtensions};

/// A game packet.
#[derive(Debug, Clone)]
pub struct Packet<T: ConnectedPacket> {
    /// Contains the packet ID and subclient IDs.
    header: Header,
    /// Actual packet data.
    content: T,
}

impl<T: ConnectedPacket> Packet<T> {
    /// Creates a new packet.
    pub const fn new(pk: T) -> Self {
        Self {
            header: Header {
                id: T::ID,
                target_subclient: 0,
                sender_subclient: 0,
            },
            content: pk,
        }
    }

    /// Sets the subclient IDs.
    pub const fn subclients(mut self, sender: u8, target: u8) -> Self {
        self.header.target_subclient = target;
        self.header.sender_subclient = sender;
        self
    }
}

impl<T: ConnectedPacket + Serialize> Serialize for Packet<T> {
    fn serialize(&self) -> VResult<Bytes> {
        let mut buffer = BytesMut::new();
        let header = self.header.serialize();
        let body = self.content.serialize()?;

        buffer.put_var_u32(header.len() as u32 + body.len() as u32);

        buffer.put(header);
        buffer.put(body);

        Ok(buffer.freeze())
    }
}
