use bytes::{Buf, BytesMut};

use crate::error::VexResult;
use crate::raknet::packets::Decodable;
use crate::vex_assert;

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

impl Decodable for RequestNetworkSettings {
    fn decode(mut buffer: BytesMut) -> VexResult<Self> {
        let protocol_version = buffer.get_u32();

        Ok(Self { protocol_version })
    }
}
