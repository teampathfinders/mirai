use bytes::{BytesMut, Bytes};
use common::{VResult, Vector3f, WriteExtensions};

use common::Serialize;
use level::Dimension;

use super::ConnectedPacket;

/// Used to transfer the client to another dimension.
#[derive(Debug, Clone)]
pub struct ChangeDimension {
    /// Dimension to transfer to.
    pub dimension: Dimension,
    /// Location to spawn at in the new position.
    pub position: Vector3f,
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
    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_var_i32(self.dimension as i32);
        buffer.put_vec3f(&self.position);
        buffer.put_bool(self.respawn);
    }
}
