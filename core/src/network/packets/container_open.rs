use util::{Serialize, BlockPosition};
use util::bytes::{BinaryWrite, MutableBuffer, size_of_varint};

use crate::network::ConnectedPacket;

pub const INVENTORY_WINDOW_ID: u8 = 0xff;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ContainerType {
    #[default]
    Inventory = 0xff
}

#[derive(Debug, Clone, Default)]
pub struct ContainerOpen {
    pub window_id: u8,
    pub container_type: ContainerType,
    pub position: BlockPosition,
    pub container_entity_unique_id: i64,
}

impl ConnectedPacket for ContainerOpen {
    const ID: u32 = 0x2e;

    fn serialized_size(&self) -> usize {
        1 + 1 + 3 * 4 + size_of_varint(self.container_entity_unique_id)
    }
}

impl Serialize for ContainerOpen {
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_u8(self.window_id)?;
        buffer.write_u8(self.container_type as u8)?;
        buffer.write_block_pos(&self.position)?;
        buffer.write_var_i64(self.container_entity_unique_id)
    }
}