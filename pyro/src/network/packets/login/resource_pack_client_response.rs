use bytes::Bytes;
use bytes::{Buf, BytesMut};

use crate::network::packets::ConnectedPacket;
use util::bail;
use util::Deserialize;
use util::ReadExtensions;
use util::{Error, Result};

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
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        Ok(match value {
            0 => Self::None,
            1 => Self::Refused,
            2 => Self::SendPacks,
            3 => Self::HaveAllPacks,
            4 => Self::Completed,
            _ => bail!(Malformed, "Invalid resource pack status: {value}"),
        })
    }
}

/// Sent in response to [`ResourcePacksInfo`](super::ResourcePacksInfo) and
/// [`ResourcePackStack`](super::ResourcePackStack).
#[derive(Debug)]
pub struct ResourcePackClientResponse {
    /// The response status.
    pub status: ResourcePackStatus,
    /// IDs of affected packs.
    pub pack_ids: Vec<String>,
}

impl ConnectedPacket for ResourcePackClientResponse {
    /// Unique ID of this packet.
    const ID: u32 = 0x08;
}

impl Deserialize for ResourcePackClientResponse {
    fn deserialize(mut buffer: Bytes) -> Result<Self> {
        let status = ResourcePackStatus::try_from(buffer.get_u8())?;
        let length = buffer.get_u16();

        let mut pack_ids = Vec::with_capacity(length as usize);
        for _ in 0..length {
            pack_ids.push(buffer.get_string()?);
        }

        Ok(Self { status, pack_ids })
    }
}
