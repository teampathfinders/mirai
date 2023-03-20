use util::{Deserialize, Result, Serialize};
use util::bytes::{BinaryReader, BinaryWrite, MutableBuffer, SharedBuffer, size_of_varint};

use crate::GameMode;
use crate::ConnectedPacket;

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

impl Deserialize<'_> for SetDefaultGameMode {
    fn deserialize(mut buffer: SharedBuffer) -> Result<Self> {
        let game_mode = GameMode::try_from(buffer.read_var_i32()?)?;

        Ok(Self { game_mode })
    }
}

impl Serialize for SetDefaultGameMode {
    fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_var_i32(self.game_mode as i32)
    }
}
