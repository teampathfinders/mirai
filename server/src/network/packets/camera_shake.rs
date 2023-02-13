use bytes::{BufMut, BytesMut};
use common::VResult;

use common::Encodable;

use super::GamePacket;

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
#[derive(Debug)]
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

impl GamePacket for CameraShake {
    const ID: u32 = 0x9f;
}

impl Encodable for CameraShake {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(4 + 4 + 1 + 1);

        buffer.put_f32_le(self.intensity);
        buffer.put_f32_le(self.duration);
        buffer.put_u8(self.shake_type as u8);
        buffer.put_u8(self.action as u8);

        Ok(buffer)
    }
}
