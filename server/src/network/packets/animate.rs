use bytes::{Buf, BytesMut};
use common::{bail, ReadExtensions, VError, VResult, WriteExtensions};

use common::Decodable;

use super::GamePacket;

/// Type of animation to perform.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AnimateAction {
    SwingArm = 1,
    StopSleep = 3,
    CriticalHit,
    MagicCriticalHit,
    RowRight = 128,
    RowLeft,
}

impl TryFrom<i32> for AnimateAction {
    type Error = VError;

    fn try_from(value: i32) -> VResult<Self> {
        Ok(match value {
            1 => Self::SwingArm,
            3 => Self::StopSleep,
            4 => Self::CriticalHit,
            5 => Self::MagicCriticalHit,
            128 => Self::RowRight,
            129 => Self::RowLeft,
            _ => bail!(BadPacket, "Invalid animation action {value}"),
        })
    }
}

impl AnimateAction {
    #[inline]
    pub fn is_rowing(&self) -> bool {
        (*self as i32) & 0x80 != 0
    }
}

#[derive(Debug)]
pub struct Animate {
    /// Type of animation to perform.
    pub action_type: AnimateAction,
    /// Runtime ID of the entity performing the animation.
    pub runtime_id: u64,
    pub rowing_time: f32,
}

impl GamePacket for Animate {
    const ID: u32 = 0x2c;
}

impl Decodable for Animate {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        let action_type = AnimateAction::try_from(buffer.get_var_i32()?)?;
        let runtime_id = buffer.get_var_u64()?;

        let rowing_time = if action_type.is_rowing() {
            buffer.get_f32()
        } else {
            0.0
        };

        Ok(Self { action_type, runtime_id, rowing_time })
    }
}
