use util::{BinaryRead, SharedBuffer};
use util::Deserialize;
use util::Result;

/// Sent by the client to request a [`NetworkSettings`](crate::bedrock::NetworkSettings) packet.
#[derive(Debug)]
pub struct RequestNetworkSettings {
    /// Minecraft network version
    pub protocol_version: u32,
}

impl RequestNetworkSettings {
    /// Unique identifier of this packet.
    pub const ID: u32 = 0xc1;
}

impl Deserialize<'_> for RequestNetworkSettings {
    fn deserialize(mut buffer: SharedBuffer) -> anyhow::Result<Self> {
        let protocol_version = buffer.read_u32_be()?;

        Ok(Self { protocol_version })
    }
}
