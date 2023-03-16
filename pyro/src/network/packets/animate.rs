
use util::{bail, Error, Result};
use util::bytes::{BinaryReader, SharedBuffer};

use util::Deserialize;

use crate::ConnectedPacket;

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
    type Error = Error;

    fn try_from(value: i32) -> Result<Self> {
        Ok(match value {
            1 => Self::SwingArm,
            3 => Self::StopSleep,
            4 => Self::CriticalHit,
            5 => Self::MagicCriticalHit,
            128 => Self::RowRight,
            129 => Self::RowLeft,
            _ => bail!(Malformed, "Invalid animation action {value}"),
        })
    }
}

impl AnimateAction {
    #[inline]
    pub const fn is_rowing(&self) -> bool {
        (*self as i32) & 0x80 != 0
    }
}

#[derive(Debug, Clone)]
pub struct Animate {
    /// Type of animation to perform.
    pub action_type: AnimateAction,
    /// Runtime ID of the entity performing the animation.
    pub runtime_id: u64,
    pub rowing_time: f32,
}

impl ConnectedPacket for Animate {
    const ID: u32 = 0x2c;
}

impl Deserialize<'_> for Animate {
    fn deserialize(mut buffer: SharedBuffer) -> Result<Self> {
        let action_type = AnimateAction::try_from(buffer.read_var_i32()?)?;
        let runtime_id = buffer.read_var_u64()?;

        let rowing_time = if action_type.is_rowing() {
            buffer.read_f32_be()?
        } else {
            0.0
        };

        Ok(Self { action_type, runtime_id, rowing_time })
    }
}
