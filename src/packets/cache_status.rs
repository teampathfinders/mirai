use bytes::{Buf, BytesMut};

use crate::error::VexResult;
use crate::packets::GamePacket;
use crate::raknet::packets::{Decodable, Encodable};

/// Sent during login to let the server know whether the client supports caching.
#[derive(Debug)]
pub struct ClientCacheStatus {
    /// Whether the client supports the client-side blob cache.
    pub support: bool,
}

impl GamePacket for ClientCacheStatus {
    const ID: u32 = 0x81;
}

impl Decodable for ClientCacheStatus {
    fn decode(mut buffer: BytesMut) -> VexResult<Self> {
        let support = buffer.get_u8() == 1;

        Ok(Self {
            support
        })
    }
}