use bytes::BytesMut;
use common::{VResult, Vector3f, WriteExtensions};

use crate::network::Encodable;

use super::GamePacket;

/// The Minecraft dimensions.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Dimension {
    /// The overworld dimension.
    Overworld,
    /// The nether dimension.
    Nether,
    /// The end dimension.
    End,
}

/// Used to transfer the client to another dimension.
#[derive(Debug)]
pub struct ChangeDimension {
    /// Dimension to transfer to.
    pub dimension: Dimension,
    /// Location to spawn at in the new position.
    pub position: Vector3f,
    /// Whether this change was triggered by a respawn.
    /// For instance, when the player is sent back to the overworld after dying in the nether.
    pub respawn: bool,
}

impl GamePacket for ChangeDimension {
    const ID: u32 = 0x3d;
}

impl Encodable for ChangeDimension {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(1 + 3 * 4 + 1);

        buffer.put_var_i32(self.dimension as i32);
        buffer.put_vec3f(&self.position);
        buffer.put_bool(self.respawn);

        Ok(buffer)
    }
}
