use util::{Deserialize, Result, Serialize};
use util::bytes::{BinaryRead, BinaryWrite, MutableBuffer, SharedBuffer, size_of_varint};

use crate::network::ConnectedPacket;
use crate::network::GameMode;

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
    fn deserialize<R>(reader: R) -> anyhow::Result<Self>
    where
        R: BinaryRead<'a> + 'a
    {
        let game_mode = GameMode::try_from(reader.read_var_i32()?)?;

        Ok(Self { game_mode })
    }
}

impl Serialize for SetDefaultGameMode {
    fn serialize<W>(&self, writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite
    {
        writer.write_var_i32(self.game_mode as i32)
    }
}
