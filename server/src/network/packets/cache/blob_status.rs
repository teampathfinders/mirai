use bytes::{Buf, BytesMut};
use common::{Decodable, ReadExtensions, VResult};
use crate::network::packets::GamePacket;

#[derive(Debug, Clone)]
pub struct CacheBlobStatus {
    /// Hashes of the blobs that the client still needs.
    pub misses: Vec<u64>,
    /// Hashes of the blobs that the client already possesses.
    /// These do not have to be sent by the server.
    pub hits: Vec<u64>
}

impl GamePacket for CacheBlobStatus {
    const ID: u32 = 0x87;
}

impl Decodable for CacheBlobStatus {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        let miss_count = buffer.get_var_u32()?;
        let hit_count = buffer.get_var_u32()?;

        let mut misses = Vec::with_capacity(miss_count as usize);
        for _ in 0..miss_count {
            misses.push(buffer.get_u64_le());
        }

        let mut hits = Vec::with_capacity(hit_count as usize);
        for _ in 0..hit_count {
            hits.push(buffer.get_u64_le());
        }

        Ok(Self {
            misses, hits
        })
    }
}

