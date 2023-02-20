use bytes::{BytesMut, Bytes};
use common::{VResult, Vector3f, Vector3i, WriteExtensions, size_of_var};

use common::Serialize;

use super::ConnectedPacket;

/// Directions a painting can face.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PaintingDirection {
    South,
    West,
    North,
    East,
}

/// Adds a painting into the game.
#[derive(Debug, Clone)]
pub struct AddPainting<'a> {
    /// Entity runtime ID of the painting.
    pub runtime_id: u64,
    /// Position of the painting.
    pub position: Vector3f,
    /// Direction the painting is facing in.
    pub direction: PaintingDirection,
    /// Painting [`name`](https://minecraft.fandom.com/wiki/Painting#Data_values).
    pub name: &'a str,
}

impl ConnectedPacket for AddPainting<'_> {
    const ID: u32 = 0x16;

    fn serialized_size(&self) -> usize {
        size_of_var(self.runtime_id as i64) +
        size_of_var(self.runtime_id) + 3 * 4 +
        size_of_var(self.direction as i32) +
        size_of_var(self.name.len() as u32) + 
        self.name.len()
    }
}

impl Serialize for AddPainting<'_> {
    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_var_i64(self.runtime_id as i64); // Unique entity ID.
        buffer.put_var_u64(self.runtime_id);
        buffer.put_vec3f(&self.position);
        buffer.put_var_i32(self.direction as i32);
        buffer.put_string(self.name);
    }
}
