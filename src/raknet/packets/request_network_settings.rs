use bytes::{Buf, BytesMut};

use crate::error::VexResult;
use crate::raknet::packets::Decodable;
use crate::vex_assert;

/// Sent by the client to request a [`NetworkSettings`](super::NetworkSettings) packet.
#[derive(Debug)]
pub struct RequestNetworkSettings {
    /// Minecraft network version
    pub protocol_version: i32,
}

impl RequestNetworkSettings {
    /// Unique identifier of this packet.
    pub const ID: u8 = 0xc1;
}

impl Decodable for RequestNetworkSettings {
    fn decode(mut buffer: BytesMut) -> VexResult<Self> {
        vex_assert!(buffer.get_u8() == Self::ID);

        buffer.advance(1);
        let protocol_version = buffer.get_i32();

        Ok(Self {
            protocol_version
        })
    }
}