use util::{BinaryWrite, MutableBuffer, VarInt};
use util::Result;
use util::Serialize;

use crate::bedrock::ConnectedPacket;

/// Sent in response to [`ChunkRadiusRequest`](crate::bedrock::ChunkRadiusRequest), to notify the client of the allowed render distance.
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
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_var_i32(self.allowed_radius)
    }
}