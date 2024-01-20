use crate::types::Dimension;
use util::{Result, Vector};
use util::{BinaryWrite};
use util::Serialize;

use crate::bedrock::ConnectedPacket;

/// Used to transfer the client to another dimension.
#[derive(Debug, Clone)]
pub struct ChangeDimension {
    /// Dimension to transfer to.
    pub dimension: Dimension,
    /// Location to spawn at in the new position.
    pub position: Vector<f32, 3>,
    /// Whether this change was triggered by a respawn.
    /// For instance, when the player is sent back to the overworld after dying in the nether.
    pub respawn: bool,
}

impl ConnectedPacket for ChangeDimension {
    const ID: u32 = 0x3d;

    fn serialized_size(&self) -> usize {
        1 + 3 * 4 + 1
    }
}

impl Serialize for ChangeDimension {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_i32(self.dimension as i32)?;
        writer.write_vecf(&self.position)?;
        writer.write_bool(self.respawn)
    }
}
