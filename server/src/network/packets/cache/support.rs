use bytes::{Buf, BytesMut};

use crate::network::packets::GamePacket;
use common::Decodable;
use common::ReadExtensions;
use common::VResult;

/// Sent during login to let the server know whether the client supports caching.
#[derive(Debug, Clone)]
pub struct CacheStatus {
    /// Whether the client supports the client-side blob cache.
    pub supports_cache: bool,
}

impl GamePacket for CacheStatus {
    const ID: u32 = 0x81;
}

impl Decodable for CacheStatus {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        let support = buffer.get_bool();

        Ok(Self { supports_cache: support })
    }
}
