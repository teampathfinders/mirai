use bytes::BytesMut;
use common::{VResult, WriteExtensions};

use crate::network::Encodable;

use super::GamePacket;

#[derive(Debug)]
pub struct ChunkRadiusReply {
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

        tracing::info!("{:x?}", buffer.as_ref());

        Ok(buffer)
    }
}
