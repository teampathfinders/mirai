use util::{bail, Error, Result};
use util::{Deserialize, Serialize};
use util::bytes::{BinaryRead, BinaryWrite, MutableBuffer, SharedBuffer, size_of_varint};

use crate::network::ConnectedPacket;

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
    type Error = anyhow::Error;

    fn try_from(value: i32) -> anyhow::Result<Self> {
        Ok(match value {
            0 => Self::Survival,
            1 => Self::Creative,
            2 => Self::Adventure,
            3 => Self::Spectator,
            5 => Self::WorldDefault,
            _ => bail!(Malformed, "Invalid game mode"),
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

    fn serialized_size(&self) -> usize {
        size_of_varint(self.game_mode as i32)
    }
}

impl Serialize for SetPlayerGameMode {
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_var_i32(self.game_mode as i32)
    }
}

impl Deserialize<'_> for SetPlayerGameMode {
    fn deserialize(mut buffer: SharedBuffer) -> anyhow::Result<Self> {
        let game_mode = GameMode::try_from(buffer.read_var_i32()?)?;

        Ok(Self { game_mode })
    }
}
