use util::{bail};
use util::{BinaryRead};
use util::Deserialize;

use crate::bedrock::ConnectedPacket;

/// Type of animation to perform.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AnimateAction {
    /// The player punched something.
    SwingArm = 1,
    /// The player stopped sleeping.
    StopSleep = 3,
    /// A critical hit.
    CriticalHit,
    /// A magic critical hit.
    MagicCriticalHit,
    /// The player is rowing to the right.
    RowRight = 128,
    /// The player is rowing to the left.
    RowLeft,
}

impl TryFrom<i32> for AnimateAction {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> anyhow::Result<Self> {
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
    /// Whether the player is currently rowing.
    #[inline]
    pub const fn is_rowing(&self) -> bool {
        (*self as i32) & 0x80 != 0
    }
}

/// Plays an animation.
#[derive(Debug, Clone)]
pub struct Animate {
    /// Type of animation to perform.
    pub action_type: AnimateAction,
    /// Runtime ID of the entity performing the animation.
    pub runtime_id: u64,
    /// How long the client has been rowing for.
    pub rowing_time: f32,
}

impl ConnectedPacket for Animate {
    const ID: u32 = 0x2c;
}

impl<'a> Deserialize<'a> for Animate {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let action_type = AnimateAction::try_from(reader.read_var_i32()?)?;
        let runtime_id = reader.read_var_u64()?;

        let rowing_time = if action_type.is_rowing() {
            reader.read_f32_be()?
        } else {
            0.0
        };

        Ok(Self { action_type, runtime_id, rowing_time })
    }
}
