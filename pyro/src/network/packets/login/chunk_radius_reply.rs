use bytes::{BytesMut, Bytes};
use util::{Result, WriteExtensions, size_of_varint, VarInt};

use util::Serialize;
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
        self.allowed_radius.var_len()
    }
}

impl Serialize for ChunkRadiusReply {
    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_var_i32(self.allowed_radius);
    }
}
