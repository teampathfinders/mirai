
use util::{
    bail, Error, Result, Vector3f
};

use util::{Deserialize, Serialize};
use util::bytes::{BinaryReader, BinaryWriter, MutableBuffer, SharedBuffer, size_of_varint};

use super::ConnectedPacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RespawnState {
    Searching,
    ServerReady,
    ClientReady,
}

impl TryFrom<u8> for RespawnState {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        Ok(match value {
            0 => Self::Searching,
            1 => Self::ServerReady,
            2 => Self::ClientReady,
            _ => bail!(Malformed, "Invalid respawn state {value}"),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Respawn {
    pub position: Vector3f,
    pub state: RespawnState,
    pub runtime_id: u64,
}

impl ConnectedPacket for Respawn {
    const ID: u32 = 0x2d;

    fn serialized_size(&self) -> usize {
        3 * 4 + 1 + size_of_varint(self.runtime_id)
    }
}

impl Serialize for Respawn {
    fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_vecf(&self.position);
        buffer.write_u8(self.state as u8);
        buffer.write_var_u64(self.runtime_id);

        Ok(())
    }
}

impl Deserialize<'_> for Respawn {
    fn deserialize(mut buffer: SharedBuffer) -> Result<Self> {
        let position = buffer.read_vecf()?;
        let state = RespawnState::try_from(buffer.read_u8()?)?;
        let runtime_id = buffer.read_var_u64()?;

        Ok(Self { position, state, runtime_id })
    }
}
