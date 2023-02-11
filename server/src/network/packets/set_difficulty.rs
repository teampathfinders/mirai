use bytes::{BytesMut, Bytes};
use common::{VResult, WriteExtensions, ReadExtensions};

use crate::network::{Encodable, Decodable};

use super::{Difficulty, GamePacket};

#[derive(Debug)]
pub struct SetDifficulty {
    pub difficulty: Difficulty
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