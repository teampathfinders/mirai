


use crate::ConnectedPacket;
use util::bail;
use util::Deserialize;
use util::{Error, Result};
use util::bytes::{BinaryReader, SharedBuffer};

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

/// Sent in response to [`ResourcePacksInfo`](crate::ResourcePacksInfo) and
/// [`ResourcePackStack`](crate::ResourcePackStack).
#[derive(Debug)]
pub struct ResourcePackClientResponse<'a> {
    /// The response status.
    pub status: ResourcePackStatus,
    /// IDs of affected packs.
    pub pack_ids: Vec<&'a str>,
}

impl<'a> ConnectedPacket for ResourcePackClientResponse<'a> {
    /// Unique ID of this packet.
    const ID: u32 = 0x08;
}

impl<'a> Deserialize<'a> for ResourcePackClientResponse<'a> {
    fn deserialize(mut buffer: SharedBuffer<'a>) -> Result<Self> {
        let status = ResourcePackStatus::try_from(buffer.read_u8()?)?;
        let length = buffer.read_u16_be()?;

        let mut pack_ids = Vec::with_capacity(length as usize);
        for _ in 0..length {
            pack_ids.push(buffer.read_str()?);
        }

        Ok(Self { status, pack_ids })
    }
}