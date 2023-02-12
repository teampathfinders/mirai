use bytes::{Buf, BufMut, BytesMut};
use common::{bail, ReadExtensions, VError, VResult, Vector3f, WriteExtensions};

use crate::network::{Decodable, Encodable};

use super::GamePacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RespawnState {
    Searching,
    ServerReady,
    ClientReady,
}

impl TryFrom<u8> for RespawnState {
    type Error = VError;

    fn try_from(value: u8) -> VResult<Self> {
        Ok(match value {
            0 => Self::Searching,
            1 => Self::ServerReady,
            2 => Self::ClientReady,
            _ => bail!(BadPacket, "Invalid respawn state {value}"),
        })
    }
}

#[derive(Debug)]
pub struct Respawn {
    pub position: Vector3f,
    pub state: RespawnState,
    pub runtime_id: u64,
}

impl GamePacket for Respawn {
    const ID: u32 = 0x2d;
}

impl Encodable for Respawn {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_vec3f(&self.position);
        buffer.put_u8(self.state as u8);
        buffer.put_var_u64(self.runtime_id);

        Ok(buffer)
    }
}

impl Decodable for Respawn {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        let position = buffer.get_vec3f();
        let state = RespawnState::try_from(buffer.get_u8())?;
        let runtime_id = buffer.get_var_u64()?;

        Ok(Self {
            position,
            state,
            runtime_id,
        })
    }
}
