
use util::{BlockPosition, Deserialize, Vector3i, Result};
use util::bytes::{BinaryReader, SharedBuffer};
use crate::network::packets::ConnectedPacket;

/// Sent by the client when the user requests a block using the block pick key.
#[derive(Debug)]
pub struct BlockPickRequest {
    /// Position of the block to pick.
    pub position: Vector3i,
    /// Whether to include the block's NBT tags.
    pub with_nbt: bool,
    /// Hot bar slot to put the item into.
    pub hotbar_slot: u8
}

impl ConnectedPacket for BlockPickRequest {
    const ID: u32 = 0x22;
}

impl Deserialize<'_> for BlockPickRequest {
    fn deserialize(mut buffer: SharedBuffer) -> Result<Self> {
        let position = buffer.read_vec3i()?;
        let with_nbt = buffer.read_bool()?;
        let hotbar_slot = buffer.read_u8()?;
        
        Ok(Self {
            position,
            with_nbt,
            hotbar_slot
        })
    }
}