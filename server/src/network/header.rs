use bytes::{BufMut, BytesMut};

use common::{size_of_var, VResult};
use common::{ReadExtensions, WriteExtensions};

/// Game packets are prefixed with a length and a header.
/// The header contains the packet ID and target/subclient IDs in case of split screen multiplayer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    /// Packet ID
    pub id: u32,
    /// Subclient ID of the sender
    pub sender_subclient: u8,
    /// Subclient ID of the target
    pub target_subclient: u8,
}

impl Header {
    /// Decodes the header.
    pub fn decode(buffer: &mut BytesMut) -> VResult<Self> {
        let value = buffer.get_var_u32()?;

        let id = value & 0x3ff;
        let sender_subclient = ((value >> 10) & 0x3) as u8;
        let target_subclient = ((value >> 12) & 0x3) as u8;

        Ok(Self { id, sender_subclient, target_subclient })
    }

    /// Encodes the header.
    pub fn encode(&self) -> BytesMut {
        let value = self.id
            | ((self.sender_subclient as u32) << 10)
            | ((self.target_subclient as u32) << 12);

        let mut buffer = BytesMut::with_capacity(size_of_var(value));
        buffer.put_var_u32(value);

        buffer
    }
}
