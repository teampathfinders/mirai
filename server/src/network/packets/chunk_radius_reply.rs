use bytes::BytesMut;
use common::{VResult, WriteExtensions};

use common::Encodable;

use super::GamePacket;

/// Sent in response to [`ChunkRadiusRequest`](super::ChunkRadiusRequest), to notify the client of the allowed render distance.
#[derive(Debug)]
pub struct ChunkRadiusReply {
    /// Maximum render distance that the server allows (in chunks).
    pub allowed_radius: i32,
}

impl GamePacket for ChunkRadiusReply {
    const ID: u32 = 0x46;
}

impl Encodable for ChunkRadiusReply {
    fn encode(&self) -> VResult<BytesMut> {
        // Chunk radius is virtually always one byte,
        // unless a chunk radius larger than 128 chunks is specified.
        let mut buffer = BytesMut::with_capacity(1);

        buffer.put_var_i32(self.allowed_radius);

        Ok(buffer)
    }
}
