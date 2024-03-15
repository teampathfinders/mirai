use util::{bail};
use util::{Deserialize, Serialize};
use util::{BinaryRead, BinaryWrite, size_of_varint};

use crate::bedrock::ConnectedPacket;

/// The Minecraft game modes.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GameMode {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    SurvivalSpectator = 3,
    CreativeSpectator = 4,
    /// Sets the player's game mode to the world default.
    WorldDefault = 5,
    Spectator = 6
}

impl TryFrom<i32> for GameMode {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> anyhow::Result<Self> {
        Ok(match value {
            0 => Self::Survival,
            1 => Self::Creative,
            2 => Self::Adventure,
            3 => Self::SurvivalSpectator,
            4 => Self::CreativeSpectator,
            5 => Self::WorldDefault,
            6 => Self::Spectator,
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
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_i32(self.game_mode as i32)
    }
}

impl<'a> Deserialize<'a> for SetPlayerGameMode {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let game_mode = GameMode::try_from(reader.read_var_i32()?)?;

        Ok(Self { game_mode })
    }
}
