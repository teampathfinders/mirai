use bytes::BytesMut;
use common::{Decodable, ReadExtensions, VResult};
use crate::network::packets::GamePacket;

#[derive(Debug, Clone)]
pub struct GenericLevelEvent {
    pub event_id: i32,
    pub data: nbt::Tag
}

impl GamePacket for GenericLevelEvent {
    const ID: u32 = 0x7c;
}

impl Decodable for GenericLevelEvent {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        let event_id = buffer.get_var_i32()?;
        let data = nbt::read_le(&mut buffer)?;

        Ok(Self {
            event_id,
            data
        })
    }
}