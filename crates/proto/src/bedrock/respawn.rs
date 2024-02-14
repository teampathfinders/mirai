use util::{bail, Vector};
use util::{Deserialize, Serialize};
use util::{BinaryRead, BinaryWrite, size_of_varint};

use crate::bedrock::ConnectedPacket;

/// State of the respawn process.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RespawnState {
    /// The server is searching for a place to spawn the client.
    Searching,
    /// The server is ready for respawning.
    ServerReady,
    /// The client is ready for respawning.
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

/// Tells a client to respawn.
#[derive(Debug, Clone)]
pub struct Respawn {
    /// Respawn position.
    pub position: Vector<f32, 3>,
    /// Respawn state.
    pub state: RespawnState,
    /// Runtime ID of the client.
    pub runtime_id: u64,
}

impl ConnectedPacket for Respawn {
    const ID: u32 = 0x2d;

    fn serialized_size(&self) -> usize {
        3 * 4 + 1 + size_of_varint(self.runtime_id)
    }
}

impl Serialize for Respawn {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_vecf(&self.position)?;
        writer.write_u8(self.state as u8)?;
        writer.write_var_u64(self.runtime_id)
    }
}

impl<'a> Deserialize<'a> for Respawn {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let position = reader.read_vecf()?;
        let state = RespawnState::try_from(reader.read_u8()?)?;
        let runtime_id = reader.read_var_u64()?;

        Ok(Self { position, state, runtime_id })
    }
}
