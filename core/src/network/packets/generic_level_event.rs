use util::{Deserialize, Result};
use util::bytes::{BinaryRead, SharedBuffer};

use crate::network::ConnectedPacket;

#[derive(Debug, Clone)]
pub struct GenericLevelEvent {
    pub event_id: i32,
    // pub data: nbt::Tag
}

impl ConnectedPacket for GenericLevelEvent {
    const ID: u32 = 0x7c;
}

impl Deserialize<'_> for GenericLevelEvent {
    fn deserialize(mut buffer: SharedBuffer) -> anyhow::Result<Self> {
        let _event_id = buffer.read_var_i32()?;
        // let data = nbt::from_le_bytes(&mut buffer)?;

        todo!();
        // Ok(Self {
        //     event_id,
        //     data
        // })
    }
}