use bytes::{Bytes, BytesMut};
use common::{ReadExtensions, VResult, WriteExtensions, VError, bail};

use crate::network::{Decodable, Encodable};

use super::GamePacket;

#[derive(Debug, Copy, Clone, PartialEq)]
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

#[derive(Debug)]
pub struct SetDifficulty {
    pub difficulty: Difficulty,
}

impl GamePacket for SetDifficulty {
    const ID: u32 = 0x3c;
}

impl Encodable for SetDifficulty {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_var_i32(self.difficulty as i32);

        Ok(buffer)
    }
}

impl Decodable for SetDifficulty {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        let difficulty = Difficulty::try_from(buffer.get_var_i32()?)?;
        Ok(Self { difficulty })
    }
}
