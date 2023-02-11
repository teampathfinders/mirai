use bytes::BytesMut;
use common::{VResult, WriteExtensions};

use crate::network::Encodable;

use super::{GameMode, GamePacket};

#[derive(Debug)]
pub struct SetPlayerGameMode {
    pub game_mode: GameMode
}

impl GamePacket for SetPlayerGameMode {
    const ID: u32 = 0x3e;
}

impl Encodable for SetPlayerGameMode {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        self.game_mode.encode(&mut buffer);

        Ok(buffer)
    }
}