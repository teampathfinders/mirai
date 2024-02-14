use util::{bail, BlockPosition, Deserialize, Serialize};
use util::{BinaryRead, BinaryWrite, size_of_varint};

use crate::bedrock::ConnectedPacket;

/// The type of block event.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BlockEventType {
    /// Changes the state of a chest.
    ChangeChestState
}

impl TryFrom<i32> for BlockEventType {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> anyhow::Result<Self> {
        Ok(match value {
            0 => Self::ChangeChestState,
            _ => bail!(Malformed, "Invalid block event type {value}")
        })
    }
}

/// A block event.
#[derive(Debug, Clone)]
pub struct BlockEvent {
    /// Position of the block event.
    pub position: BlockPosition,
    /// The type of block event.
    pub event_type: BlockEventType,
    /// Associated block event data.
    pub event_data: i32,
}

impl ConnectedPacket for BlockEvent {
    const ID: u32 = 0x1a;

    fn serialized_size(&self) -> usize {
        size_of_varint(self.position.x) +
            size_of_varint(self.position.y) +
            size_of_varint(self.position.z) +
            size_of_varint(self.event_type as i32) +
            size_of_varint(self.event_data)
    }
}

impl Serialize for BlockEvent {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_block_pos(&self.position)?;
        writer.write_var_i32(self.event_type as i32)?;
        writer.write_var_i32(self.event_data)
    }
}

impl<'a> Deserialize<'a> for BlockEvent {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let position = reader.read_block_pos()?;
        let event_type = BlockEventType::try_from(reader.read_var_i32()?)?;
        let event_data = reader.read_var_i32()?;

        Ok(Self {
            position,
            event_type,
            event_data,
        })
    }
}