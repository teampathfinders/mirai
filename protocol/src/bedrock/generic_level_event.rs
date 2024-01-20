use util::{Deserialize, Result};
use util::{BinaryRead, SharedBuffer};

use crate::bedrock::ConnectedPacket;

#[derive(Debug, Clone)]
pub struct GenericLevelEvent {
    pub event_id: i32,
    // pub data: nbt::Tag
}

impl ConnectedPacket for GenericLevelEvent {
    const ID: u32 = 0x7c;
}

impl<'a> Deserialize<'a> for GenericLevelEvent {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let _event_id = reader.read_var_i32()?;
        // let data = nbt::from_le_bytes(&mut buffer)?;

        todo!();
        // Ok(Self {
        //     event_id,
        //     data
        // })
    }
}