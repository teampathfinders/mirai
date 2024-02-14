use util::{BinaryRead, Deserialize};

/// Sent by the client to request a [`NetworkSettings`](crate::bedrock::NetworkSettings) packet.
#[derive(Debug)]
pub struct RequestNetworkSettings {
    /// Minecraft network version. In case this version does not match the server's version,
    /// the client is disconnected using a [`PlayStatus`](crate::bedrock::PlayStatus) packet.
    pub protocol_version: u32,
}

impl RequestNetworkSettings {
    /// Unique identifier of this packet.
    pub const ID: u32 = 0xc1;
}

impl<'a> Deserialize<'a> for RequestNetworkSettings {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let protocol_version = reader.read_u32_be()?;

        Ok(Self { protocol_version })
    }
}
