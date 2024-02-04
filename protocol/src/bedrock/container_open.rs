use util::{Serialize, BlockPosition};
use util::{BinaryWrite, size_of_varint};

use crate::bedrock::ConnectedPacket;

/// ID of the special inventory container.
pub const INVENTORY_WINDOW_ID: u8 = 0xff;

/// Type of container to open.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ContainerType {
    /// The inventory container type.
    #[default]
    Inventory = 0xff
}

/// Sent when a container has been opened.
#[derive(Debug, Clone, Default)]
pub struct ContainerOpen {
    /// ID of the container window.
    pub window_id: u8,
    /// Type of the container.
    pub container_type: ContainerType,
    /// Position of the container.
    pub position: BlockPosition,
    /// Unique ID of the entity if the container is an entity.
    pub container_entity_unique_id: i64,
}

impl ConnectedPacket for ContainerOpen {
    const ID: u32 = 0x2e;

    fn serialized_size(&self) -> usize {
        1 + 1 + 3 * 4 + size_of_varint(self.container_entity_unique_id)
    }
}

impl Serialize for ContainerOpen {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_u8(self.window_id)?;
        writer.write_u8(self.container_type as u8)?;
        writer.write_block_pos(&self.position)?;
        writer.write_var_i64(self.container_entity_unique_id)
    }
}