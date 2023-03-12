use bytes::{BytesMut, Bytes};
use common::{Deserialize, Serialize, ReadExtensions, Result, WriteExtensions, size_of_varint};

use crate::network::packets::ConnectedPacket;

use super::{GameMode};

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

impl Deserialize for SetDefaultGameMode {
    fn deserialize(mut buffer: Bytes) -> Result<Self> {
        let game_mode = GameMode::try_from(buffer.get_var_i32()?)?;

        Ok(Self { game_mode })
    }
}

impl Serialize for SetDefaultGameMode {
    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_var_i32(self.game_mode as i32);
    }
}
