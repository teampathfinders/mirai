
use util::{bail, Error, Result};

use util::{Deserialize, Serialize};
use util::bytes::{BinaryReader, BinaryWrite, MutableBuffer, SharedBuffer, size_of_varint};

use crate::ConnectedPacket;

/// The Minecraft difficulties.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl TryFrom<i32> for Difficulty {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self> {
        Ok(match value {
            0 => Self::Peaceful,
            1 => Self::Easy,
            2 => Self::Normal,
            3 => Self::Hard,
            _ => bail!(Malformed, "Invalid difficulty type {value}"),
        })
    }
}

/// Sets the difficulty of the level.
///
/// This does not do a lot client-side, it is mainly used to sync the difficulty setting in the client's world settings.
#[derive(Debug, Clone)]
pub struct SetDifficulty {
    /// Difficulty to apply.
    pub difficulty: Difficulty,
}

impl ConnectedPacket for SetDifficulty {
    const ID: u32 = 0x3c;

    fn serialized_size(&self) -> usize {
        size_of_varint(self.difficulty as i32)
    }
}

impl Serialize for SetDifficulty {
    fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_var_i32(self.difficulty as i32);

        Ok(())
    }
}

impl Deserialize<'_> for SetDifficulty {
    fn deserialize(mut buffer: SharedBuffer) -> Result<Self> {
        let difficulty = Difficulty::try_from(buffer.read_var_i32()?)?;

        Ok(Self { difficulty })
    }
}
