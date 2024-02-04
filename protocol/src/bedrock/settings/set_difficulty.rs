use util::{bail};
use util::{Deserialize, Serialize};
use util::{BinaryRead, BinaryWrite, size_of_varint};

use crate::bedrock::ConnectedPacket;

/// The Minecraft difficulties.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl TryFrom<i32> for Difficulty {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> anyhow::Result<Self> {
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
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_i32(self.difficulty as i32)
    }
}

impl<'a> Deserialize<'a> for SetDifficulty {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let difficulty = Difficulty::try_from(reader.read_var_i32()?)?;

        Ok(Self { difficulty })
    }
}
