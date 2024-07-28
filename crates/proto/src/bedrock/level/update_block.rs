use util::{BinaryWrite, BlockPosition, Serialize};

use crate::bedrock::ConnectedPacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum UpdateBlockFlags {
    UpdateNeighbors = 1 << 1,
    UpdateNetwork = 1 << 2,
    UpdateNoGraphics = 1 << 3,
    UpdatePriority = 1 << 4
}

/// Updates a single block in a chunk rather than sending the entire chunk.
#[derive(Debug, Clone)]
pub struct UpdateBlock {
    /// Position to place the block at.
    pub position: BlockPosition,
    /// The runtime ID of the new block.
    pub block_runtime_id: u32,
    /// Flags that specify the way the block is updated.
    pub flags: u32,
    /// Layer of the world that is updated. This layer concept is the same as seen in subchunk storage layers.
    pub layer: u32
}

impl ConnectedPacket for UpdateBlock {
    const ID: u32 = 0x15;
}

impl Serialize for UpdateBlock {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_block_pos(&self.position)?;
        writer.write_var_u32(self.block_runtime_id)?;
        writer.write_var_u32(self.flags)?;
        writer.write_var_u32(self.layer)
    }
}

