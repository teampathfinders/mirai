use bytes::{Buf, BytesMut};

use common::nvassert;
use common::Deserialize;
use common::VResult;

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
    fn deserialize(mut buffer: BytesMut) -> VResult<Self> {
        let protocol_version = buffer.get_u32();

        Ok(Self { protocol_version })
    }
}
