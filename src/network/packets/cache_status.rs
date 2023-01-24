use bytes::{Buf, BytesMut};

use crate::error::VexResult;
use crate::network::packets::GamePacket;
use crate::network::traits::Decodable;
use crate::util::ReadExtensions;

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
        let support = buffer.get_bool();

        Ok(Self { support })
    }
}
