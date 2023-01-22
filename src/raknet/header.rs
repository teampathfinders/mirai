use bytes::BytesMut;

use crate::error::VexResult;
use crate::util::ReadExtensions;

/// Game packets are prefixed with a length and a header.
/// The header contains the packet ID and target/subclient IDs in case of splitscreen multiplayer.
#[derive(Debug)]
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
    pub fn decode(buffer: &mut BytesMut) -> VexResult<Self> {
        let value = buffer.get_var_u32()?;

        let id = value & 0x3ff;
        let sender_subclient = ((value >> 10) & 0x3) as u8;
        let target_subclient = ((value >> 12) & 0x3) as u8;

        Ok(Self {
            id,
            sender_subclient,
            target_subclient,
        })
    }

    pub fn encode(&self, buffer: &mut BytesMut) {
        todo!()
    }
}
