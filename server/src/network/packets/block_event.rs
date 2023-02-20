use bytes::{BytesMut, Bytes};
use common::{bail, BlockPosition, Deserialize, Serialize, ReadExtensions, VError, VResult, WriteExtensions, size_of_varint};
use crate::network::packets::ConnectedPacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BlockEventType {
    ChangeChestState
}

impl TryFrom<i32> for BlockEventType {
    type Error = VError;

    fn try_from(value: i32) -> VResult<Self> {
        Ok(match value {
            0 => Self::ChangeChestState,
            _ => bail!(BadPacket, "Invalid block event type {value}")
        })
    }
}

#[derive(Debug, Clone)]
pub struct BlockEvent {
    pub position: BlockPosition,
    pub event_type: BlockEventType,
    pub event_data: i32
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
    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_block_pos(&self.position);
        buffer.put_var_i32(self.event_type as i32);
        buffer.put_var_i32(self.event_data);
    }
}

impl Deserialize for BlockEvent {
    fn deserialize(mut buffer: Bytes) -> VResult<Self> {
        let position = buffer.get_block_pos()?;
        let event_type = BlockEventType::try_from(buffer.get_var_i32()?)?;
        let event_data = buffer.get_var_i32()?;

        Ok(Self {
            position, event_type, event_data
        })
    }
}