use util::{Result, Vector};
use util::bytes::{BinaryWrite, MutableBuffer, size_of_varint};
use util::Serialize;

use crate::network::ConnectedPacket;

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
    pub position: Vector<f32, 3>,
    /// Direction the painting is facing in.
    pub direction: PaintingDirection,
    /// Painting [`name`](https://minecraft.fandom.com/wiki/Painting#Data_values).
    pub name: &'a str,
}

impl<'a> ConnectedPacket for AddPainting<'a> {
    const ID: u32 = 0x16;

    fn serialized_size(&self) -> usize {
        size_of_varint(self.runtime_id as i64) +
            size_of_varint(self.runtime_id) + 3 * 4 +
            size_of_varint(self.direction as i32) +
            size_of_varint(self.name.len() as u32) +
            self.name.len()
    }
}

impl<'a> Serialize for AddPainting<'a> {
    fn serialize(&self, writer: impl BinaryWrite) -> anyhow::Result<()> {
        writer.write_var_i64(self.runtime_id as i64)?; // Unique entity ID.
        writer.write_var_u64(self.runtime_id)?;
        writer.write_vecf(&self.position)?;
        writer.write_var_i32(self.direction as i32)?;
        writer.write_str(self.name)
    }
}
