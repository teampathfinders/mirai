use util::bytes::{BinaryRead, SharedBuffer};
use util::Deserialize;
use util::Result;

/// Sent by the client to request a [`NetworkSettings`](crate::NetworkSettings) packet.
#[derive(Debug)]
pub struct RequestNetworkSettings {
    /// Minecraft network version
    pub protocol_version: u32,
}

impl RequestNetworkSettings {
    /// Unique identifier of this packet.
    pub const ID: u32 = 0xc1;
}

impl<'a> Deserialize<'a> for RequestNetworkSettings {
    fn deserialize<R>(reader: R) -> anyhow::Result<Self>
    where
        R: BinaryRead<'a> + 'a
    {
        let protocol_version = reader.read_u32_be()?;

        Ok(Self { protocol_version })
    }
}
