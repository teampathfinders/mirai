use bytes::BytesMut;
use common::{bail, BlockPosition, Decodable, Encodable, ReadExtensions, VError, VResult, WriteExtensions};
use crate::network::packets::GamePacket;

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

impl GamePacket for BlockEvent {
    const ID: u32 = 0x1a;
}

impl Encodable for BlockEvent {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_block_pos(&self.position);
        buffer.put_var_i32(self.event_type as i32);
        buffer.put_var_i32(self.event_data);

        Ok(buffer)
    }
}

impl Decodable for BlockEvent {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        let position = buffer.get_block_pos();
        let event_type = BlockEventType::try_from(buffer.get_var_i32()?)?;
        let event_data = buffer.get_var_i32()?;

        Ok(Self {
            position, event_type, event_data
        })
    }
}