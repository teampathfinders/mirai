use bytes::{BytesMut, Bytes};
use common::{VResult, WriteExtensions, size_of_var};

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

    fn serialized_size(&self) -> usize {
        size_of_var(self.allowed_radius)
    }
}

impl Serialize for ChunkRadiusReply {
    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_var_i32(self.allowed_radius);
    }
}
