use bytes::BytesMut;
use common::{VResult, WriteExtensions};

use common::Serialize;
use crate::network::packets::ConnectedPacket;

/// Sent in response to [`ChunkRadiusRequest`](super::ChunkRadiusRequest), to notify the client of the allowed render distance.
#[derive(Debug, Clone)]
pub struct ChunkRadiusReply {
    /// Maximum render distance that the server allows (in chunks).
    pub allowed_radius: i32,
}

impl ConnectedPacket for ChunkRadiusReply {
    const ID: u32 = 0x46;
}

impl Serialize for ChunkRadiusReply {
    fn serialize(&self) -> VResult<BytesMut> {
        // Chunk radius is virtually always one byte,
        // unless a chunk radius larger than 128 chunks is specified.
        let mut buffer = BytesMut::with_capacity(1);

        buffer.put_var_i32(self.allowed_radius);

        Ok(buffer)
    }
}
