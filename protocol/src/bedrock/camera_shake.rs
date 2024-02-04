use util::{BinaryWrite};

use util::Serialize;

use crate::bedrock::ConnectedPacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CameraShakeType {
    Positional,
    Rotational,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CameraShakeAction {
    Add,
    Remove,
}

/// Makes the camera shake client-side.
/// This can be used for map-making.
#[derive(Debug, Clone)]
pub struct CameraShake {
    /// Intensity.
    pub intensity: f32,
    /// Duration.
    pub duration: f32,
    /// Type of the shake.
    pub shake_type: CameraShakeType,
    /// Type of the action.
    pub action: CameraShakeAction,
}

impl ConnectedPacket for CameraShake {
    const ID: u32 = 0x9f;

    fn serialized_size(&self) -> usize {
        4 + 4 + 1 + 1
    }
}

impl Serialize for CameraShake {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_f32_le(self.intensity)?;
        writer.write_f32_le(self.duration)?;
        writer.write_u8(self.shake_type as u8)?;
        writer.write_u8(self.action as u8)
    }
}
