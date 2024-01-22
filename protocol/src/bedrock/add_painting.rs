use util::{Result, Vector};
use util::{BinaryWrite, size_of_varint};
use util::Serialize;

use crate::bedrock::ConnectedPacket;

/// Directions a painting can face.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(i32)]
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
    /// Painting [`name`](https://minecraft.wiki/w/Painting#Canvases).
    pub name: &'a str,
}

impl ConnectedPacket for AddPainting<'_> {
    const ID: u32 = 0x16;

    fn serialized_size(&self) -> usize {
        size_of_varint(self.runtime_id as i64) +
            size_of_varint(self.runtime_id) + 3 * 4 +
            size_of_varint(self.direction as i32) +
            size_of_varint(self.name.len() as u32) +
            self.name.len()
    }
}

impl Serialize for AddPainting<'_> {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_i64(self.runtime_id as i64)?; // Unique entity ID.
        writer.write_var_u64(self.runtime_id)?;
        writer.write_vecf(&self.position)?;
        writer.write_var_i32(self.direction as i32)?;
        writer.write_str(self.name)
    }
}
