use util::{Deserialize};
use util::{BinaryRead};

use crate::bedrock::ConnectedPacket;

/// A generic level event.
/// 
/// The data of this event is encoded in NBT form.
#[derive(Debug)]
pub struct GenericLevelEvent {
    /// ID of the generic level event.
    pub event_id: i32,
    /// Extra data for the event.
    pub data: nbt::Value
}

impl ConnectedPacket for GenericLevelEvent {
    const ID: u32 = 0x7c;
}

impl<'a> Deserialize<'a> for GenericLevelEvent {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let event_id = reader.read_var_i32()?;
        let (data, _) = nbt::from_le_bytes(reader)?;

        Ok(Self {
            event_id,
            data
        })
    }
}