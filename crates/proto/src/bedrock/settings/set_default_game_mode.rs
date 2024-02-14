use util::{Deserialize, Serialize};
use util::{BinaryRead, BinaryWrite, size_of_varint};

use crate::bedrock::ConnectedPacket;
use crate::bedrock::GameMode;

/// Sets the default game mode of the world.
#[derive(Debug, Clone)]
pub struct SetDefaultGameMode {
    /// Game mode.
    pub game_mode: GameMode,
}

impl ConnectedPacket for SetDefaultGameMode {
    const ID: u32 = 0x69;

    fn serialized_size(&self) -> usize {
        size_of_varint(self.game_mode as i32)
    }
}

impl<'a> Deserialize<'a> for SetDefaultGameMode {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let game_mode = GameMode::try_from(reader.read_var_i32()?)?;

        Ok(Self { game_mode })
    }
}

impl Serialize for SetDefaultGameMode {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_i32(self.game_mode as i32)
    }
}
