use bytes::{BytesMut, Bytes};
use util::{Deserialize, Result};
use util::bytes::{BinaryReader, SharedBuffer};
use crate::network::packets::ConnectedPacket;

#[derive(Debug, Clone)]
pub struct GenericLevelEvent {
    pub event_id: i32,
    pub data: nbt::Tag
}

impl ConnectedPacket for GenericLevelEvent {
    const ID: u32 = 0x7c;
}

impl Deserialize for GenericLevelEvent {
    fn deserialize(mut buffer: SharedBuffer) -> Result<Self> {
        let event_id = buffer.read_var_i32()?;
        let data = nbt::from_le_bytes(&mut buffer)?;

        Ok(Self {
            event_id,
            data
        })
    }
}