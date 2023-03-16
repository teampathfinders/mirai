use std::num::NonZeroU64;




use crate::Header;
use crate::ConnectedPacket;
use util::{pyassert};
use util::bytes::{BinaryWriter, MutableBuffer, SharedBuffer};
use util::Serialize;
use util::Result;

/// A game packet.
#[derive(Debug, Clone)]
pub struct Packet<T: ConnectedPacket> {
    /// Contains the packet ID and subclient IDs.
    header: Header,
    /// Actual packet data.
    content: T,
}

impl<T: ConnectedPacket + Serialize> Packet<T> {
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

    pub fn serialized_size(&self) -> usize {
        self.header.serialized_size() + self.content.serialized_size()
    }

    pub fn serialize(&self) -> MutableBuffer {
        let expected_size = self.header.serialized_size() + self.content.serialized_size();
        let capacity = 5 + expected_size;

        // FIXME: Properly calculate packet size to reduce this to one allocation.
        let mut buffer = MutableBuffer::with_capacity(capacity);

        let mut rest = MutableBuffer::with_capacity(expected_size);
        self.header.serialize(&mut rest);
        self.content.serialize(&mut rest);

        // debug_assert_eq!(rest.len(), expected_size, "While serializing {:x}", self.header.id);

        buffer.write_var_u32(rest.len() as u32);
        buffer.append(&rest);

        buffer
    }
}