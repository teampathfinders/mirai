use level::Dimension;
use util::{Result, Vector};
use util::bytes::{BinaryWrite, MutableBuffer};
use util::Serialize;

use crate::ConnectedPacket;

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
    fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_var_i32(self.dimension as i32)?;
        buffer.write_vecf(&self.position)?;
        buffer.write_bool(self.respawn)
    }
}
