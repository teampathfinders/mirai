use bytes::BytesMut;
use common::{VResult, WriteExtensions, ReadExtensions};

use crate::network::{Encodable, Decodable};

use super::{GameMode, GamePacket};

#[derive(Debug)]
pub struct SetPlayerGameMode {
    pub game_mode: GameMode,
}

impl GamePacket for SetPlayerGameMode {
    const ID: u32 = 0x3e;
}

impl Encodable for SetPlayerGameMode {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_var_i32(self.game_mode as i32);

        Ok(buffer)
    }
}

impl Decodable for SetPlayerGameMode {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        let game_mode = GameMode::try_from(buffer.get_var_i32()?)?;
        Ok(Self { game_mode })
    }
}
