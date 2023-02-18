use bytes::{Buf, BytesMut};

use crate::network::packets::GamePacket;
use common::bail;
use common::Deserialize;
use common::ReadExtensions;
use common::{VError, VResult};

/// Status contained in [`ResourcePackClientResponse`].
#[derive(Debug, Copy, Clone)]
pub enum ResourcePackStatus {
    /// No status.
    None,
    /// Refused to download the packs.
    Refused,
    /// Client is requesting packs to be sent.
    SendPacks,
    /// Already has all packs downloaded.
    /// This is also sent when the server has no resource packs.
    HaveAllPacks,
    /// The resource pack exchange has been completed.
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
            _ => bail!(BadPacket, "Invalid resource pack status: {value}"),
        })
    }
}

/// Sent in response to [`ResourcePacksInfo`](super::ResourcePacksInfo) and
/// [`ResourcePackStack`](super::ResourcePackStack).
#[derive(Debug, Clone)]
pub struct ResourcePackClientResponse {
    /// The response status.
    pub status: ResourcePackStatus,
    /// IDs of affected packs.
    pub pack_ids: Vec<String>,
}

impl GamePacket for ResourcePackClientResponse {
    /// Unique ID of this packet.
    const ID: u32 = 0x08;
}

impl Deserialize for ResourcePackClientResponse {
    fn deserialize(mut buffer: BytesMut) -> VResult<Self> {
        let status = ResourcePackStatus::try_from(buffer.get_u8())?;
        let length = buffer.get_u16();

        let mut pack_ids = Vec::with_capacity(length as usize);
        for _ in 0..length {
            pack_ids.push(buffer.get_string()?);
        }

        Ok(Self { status, pack_ids })
    }
}
