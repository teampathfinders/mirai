use util::{Result, Serialize, Vector};
use util::bytes::{BinaryWrite, MutableBuffer, size_of_varint};

use crate::network::ConnectedPacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DebugRendererAction {
    Clear = 1,
    AddCube,
}

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
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_i32_le(self.action as i32)?;
        if self.action == DebugRendererAction::AddCube {
            buffer.write_str(self.text)?;
            buffer.write_vecf(&self.position)?;
            buffer.write_vecf(&self.color)?;
            buffer.write_i64_le(self.duration)?;
        }

        Ok(())
    }
}
