use util::{Serialize, Deserialize};
use util::{BinaryRead, BinaryWrite, size_of_varint};

/// Game raknet are prefixed with a length and a header.
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

impl Serialize for Header {
    fn size_hint(&self) -> Option<usize> {
        let value = self.id
        | ((self.sender_subclient as u32) << 10)
        | ((self.target_subclient as u32) << 12);

        Some(size_of_varint(value))
    }

    /// Encodes the header.
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        let value = self.id
            | ((self.sender_subclient as u32) << 10)
            | ((self.target_subclient as u32) << 12);

        writer.write_var_u32(value)
    }
}

impl<'a> Deserialize<'a> for Header {
    /// Decodes the header.
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let value = reader.read_var_u32()?;

        let id = value & 0x3ff;
        let sender_subclient = ((value >> 10) & 0x3) as u8;
        let target_subclient = ((value >> 12) & 0x3) as u8;

        Ok(Self { id, sender_subclient, target_subclient })
    }
}
