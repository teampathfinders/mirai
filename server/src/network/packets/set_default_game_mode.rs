use bytes::BytesMut;
use common::{Decodable, VResult, ReadExtensions, Encodable, WriteExtensions};

use super::{GameMode, GamePacket};

#[derive(Debug)]
pub struct SetDefaultGameMode {
    pub game_mode: GameMode
}

impl GamePacket for SetDefaultGameMode {
    const ID: u32 = 0x69;
}

impl Decodable for SetDefaultGameMode {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        let game_mode = GameMode::try_from(buffer.get_var_i32()?)?;

        Ok(Self { game_mode })
    }
}

impl Encodable for SetDefaultGameMode {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(1);

        buffer.put_var_i32(self.game_mode as i32);

        Ok(buffer)
    }
}