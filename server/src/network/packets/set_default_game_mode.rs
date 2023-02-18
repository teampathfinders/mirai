use bytes::BytesMut;
use common::{Deserialize, Serialize, ReadExtensions, VResult, WriteExtensions};

use super::{GameMode, GamePacket};

/// Sets the default game mode of the world.
#[derive(Debug, Clone)]
pub struct SetDefaultGameMode {
    /// Game mode.
    pub game_mode: GameMode,
}

impl GamePacket for SetDefaultGameMode {
    const ID: u32 = 0x69;
}

impl Deserialize for SetDefaultGameMode {
    fn deserialize(mut buffer: BytesMut) -> VResult<Self> {
        let game_mode = GameMode::try_from(buffer.get_var_i32()?)?;

        Ok(Self { game_mode })
    }
}

impl Serialize for SetDefaultGameMode {
    fn serialize(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(1);

        buffer.put_var_i32(self.game_mode as i32);

        Ok(buffer)
    }
}
