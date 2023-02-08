use bytes::{Buf, BytesMut};

use crate::bail;
use crate::error::{VError, VResult};
use crate::network::Decodable;
use crate::network::packets::GamePacket;
use crate::util::ReadExtensions;

#[derive(Debug, Copy, Clone)]
pub enum ResourcePackStatus {
    None,
    Refused,
    SendPacks,
    HaveAllPacks,
    Completed,
}

impl TryFrom<u8> for ResourcePackStatus {
    type Error = VError;

    fn try_from(value: u8) -> VResult<Self> {
        Ok(match value {
            0 => Self::None,
            1 => Self::Refused,
            2 => Self::SendPacks,
            3 => Self::HaveAllPacks,
            4 => Self::Completed,
            _ => bail!(BadPacket, "Invalid resource pack status: {value}")
        })
    }
}

#[derive(Debug)]
pub struct ResourcePackClientResponse {
    pub status: ResourcePackStatus,
    pub pack_ids: Vec<String>,
}

impl GamePacket for ResourcePackClientResponse {
    /// Unique ID of this packet.
    const ID: u32 = 0x08;
}

impl Decodable for ResourcePackClientResponse {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        let status = ResourcePackStatus::try_from(buffer.get_u8())?;
        let length = buffer.get_u16();

        let mut pack_ids = Vec::with_capacity(length as usize);
        for _ in 0..length {
            pack_ids.push(buffer.get_string()?);
        }

        Ok(Self {
            status,
            pack_ids,
        })
    }
}