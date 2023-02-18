use bytes::{Buf, BytesMut};
use common::{BlockPosition, Deserialize, ReadExtensions, Vector3i, VResult};
use crate::network::packets::GamePacket;

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

impl GamePacket for BlockPickRequest {
    const ID: u32 = 0x22;
}

impl Deserialize for BlockPickRequest {
    fn deserialize(mut buffer: BytesMut) -> VResult<Self> where Self: Sized {
        let position = buffer.get_vec3i();
        let with_nbt = buffer.get_bool();
        let hotbar_slot = buffer.get_u8();

        Ok(Self {
            position,
            with_nbt,
            hotbar_slot
        })
    }
}