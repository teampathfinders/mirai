use bytes::BytesMut;
use common::{ReadExtensions, VResult, WriteExtensions, VError, bail};

use crate::network::{Decodable, Encodable};

use super::GamePacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GameMode {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
    /// Sets the player's game mode to the world default.
    WorldDefault = 5,
}

impl TryFrom<i32> for GameMode {
    type Error = VError;

    fn try_from(value: i32) -> VResult<Self> {
        Ok(match value {
            0 => Self::Survival,
            1 => Self::Creative,
            2 => Self::Adventure,
            3 => Self::Spectator,
            5 => Self::WorldDefault,
            _ => bail!(BadPacket, "Invalid game mode"),
        })
    }
}

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
