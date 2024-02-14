use util::{Serialize, Vector};
use util::{BinaryWrite, size_of_varint};

use crate::bedrock::ConnectedPacket;

/// Action to perform on the debug renderer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DebugRendererAction {
    /// Removes the renderer.
    Clear = 1,
    /// Adds a cube.
    AddCube,
}

/// Creates a client-bound debug renderer.
#[derive(Debug, Clone)]
pub struct ClientBoundDebugRenderer<'a> {
    /// Action to perform.
    pub action: DebugRendererAction,
    /// Text to display above the debug renderer.
    pub text: &'a str,
    /// Position of the renderer.
    pub position: Vector<f32, 3>,
    /// Colour of the debug renderer.
    /// Every component should range from 0-1.
    pub color: Vector<f32, 4>,
    /// How long the renderer will last in milliseconds.
    pub duration: i64,
}

impl ConnectedPacket for ClientBoundDebugRenderer<'_> {
    const ID: u32 = 0xa4;

    fn serialized_size(&self) -> usize {
        4 + if self.action == DebugRendererAction::AddCube {
            size_of_varint(self.text.len() as u32) + self.text.len() +
                3 * 4 + 4 * 4 + 8
        } else {
            0
        }
    }
}

impl Serialize for ClientBoundDebugRenderer<'_> {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_i32_le(self.action as i32)?;
        if self.action == DebugRendererAction::AddCube {
            writer.write_str(self.text)?;
            writer.write_vecf(&self.position)?;
            writer.write_vecf(&self.color)?;
            writer.write_i64_le(self.duration)?;
        }

        Ok(())
    }
}
