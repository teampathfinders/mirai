use bytes::Bytes;
use bytes::{Buf, BytesMut};
use util::bytes::SharedBuffer;

use util::nvassert;
use util::Deserialize;
use util::Result;

/// Sent by the client to request a [`NetworkSettings`](super::NetworkSettings) packet.
#[derive(Debug)]
pub struct RequestNetworkSettings {
    /// Minecraft network version
    pub protocol_version: u32,
}

impl RequestNetworkSettings {
    /// Unique identifier of this packet.
    pub const ID: u32 = 0xc1;
}

impl Deserialize for RequestNetworkSettings {
    fn deserialize(mut buffer: SharedBuffer) -> Result<Self> {
        let protocol_version = buffer.read_be::<u32>()?;

        Ok(Self { protocol_version })
    }
}
