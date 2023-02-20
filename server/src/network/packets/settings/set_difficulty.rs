use bytes::{Bytes, BytesMut};
use common::{bail, ReadExtensions, VError, VResult, WriteExtensions};

use common::{Deserialize, Serialize};

use crate::network::packets::ConnectedPacket;

/// The Minecraft difficulties.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl TryFrom<i32> for Difficulty {
    type Error = VError;

    fn try_from(value: i32) -> VResult<Self> {
        Ok(match value {
            0 => Self::Peaceful,
            1 => Self::Easy,
            2 => Self::Normal,
            3 => Self::Hard,
            _ => bail!(BadPacket, "Invalid difficulty type {value}"),
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
}

impl Serialize for SetDifficulty {
    fn serialize(&self) -> VResult<Bytes> {
        let mut buffer = BytesMut::with_capacity(1);

        buffer.put_var_i32(self.difficulty as i32);

        Ok(buffer.freeze())
    }
}

impl Deserialize for SetDifficulty {
    fn deserialize(mut buffer: Bytes) -> VResult<Self> {
        let difficulty = Difficulty::try_from(buffer.get_var_i32()?)?;
        Ok(Self { difficulty })
    }
}