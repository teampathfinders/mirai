use bytes::{BytesMut, Bytes};
use common::{bail, ReadExtensions, VError, VResult, WriteExtensions};

use common::{Deserialize, Serialize};

use super::ConnectedPacket;

/// The Minecraft game modes.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
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

/// Sets the player's game mode.
#[derive(Debug, Clone)]
pub struct SetPlayerGameMode {
    /// Game mode to apply.
    pub game_mode: GameMode,
}

impl ConnectedPacket for SetPlayerGameMode {
    const ID: u32 = 0x3e;
}

impl Serialize for SetPlayerGameMode {
    fn serialize(&self) -> VResult<Bytes> {
        let mut buffer = BytesMut::with_capacity(1);

        buffer.put_var_i32(self.game_mode as i32);

        Ok(buffer.freeze())
    }
}

impl Deserialize for SetPlayerGameMode {
    fn deserialize(mut buffer: Bytes) -> VResult<Self> {
        let game_mode = GameMode::try_from(buffer.get_var_i32()?)?;
        Ok(Self { game_mode })
    }
}
