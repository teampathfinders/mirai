use util::{bail, Error, Result, Vector};
use util::{Deserialize, Serialize};
use util::bytes::{BinaryRead, BinaryWrite, MutableBuffer, SharedBuffer, size_of_varint};

use crate::network::ConnectedPacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RespawnState {
    Searching,
    ServerReady,
    ClientReady,
}

impl TryFrom<u8> for RespawnState {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> anyhow::Result<Self> {
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
    pub position: Vector<f32, 3>,
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
    fn serialize<W>(&self, writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite
    {
        writer.write_vecf(&self.position)?;
        writer.write_u8(self.state as u8)?;
        writer.write_var_u64(self.runtime_id)
    }
}

impl<'a> Deserialize<'a> for Respawn {
    fn deserialize<R>(reader: R) -> anyhow::Result<Self>
    where
        R: BinaryRead<'a> + 'a
    {
        let position = reader.read_vecf()?;
        let state = RespawnState::try_from(reader.read_u8()?)?;
        let runtime_id = reader.read_var_u64()?;

        Ok(Self { position, state, runtime_id })
    }
}
