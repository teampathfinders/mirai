use util::{Result, Serialize};
use util::bytes::{BinaryRead, BinaryWrite, MutableBuffer, SharedBuffer, size_of_varint};

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
    pub fn serialized_size(&self) -> usize {
        let value = self.id
            | ((self.sender_subclient as u32) << 10)
            | ((self.target_subclient as u32) << 12);

        size_of_varint(value)
    }
}

impl Serialize for Header {
    /// Encodes the header.
    fn serialize<W>(&self, writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite
    {
        let value = self.id
            | ((self.sender_subclient as u32) << 10)
            | ((self.target_subclient as u32) << 12);

        writer.write_var_u32(value)
    }
}

impl Header {
    /// Decodes the header.
    pub fn deserialize<'a, R>(reader: R) -> anyhow::Result<Self>
    where
        R: BinaryRead<'a> + 'a
    {
        let value = reader.read_var_u32()?;

        let id = value & 0x3ff;
        let sender_subclient = ((value >> 10) & 0x3) as u8;
        let target_subclient = ((value >> 12) & 0x3) as u8;

        Ok(Self { id, sender_subclient, target_subclient })
    }
}
