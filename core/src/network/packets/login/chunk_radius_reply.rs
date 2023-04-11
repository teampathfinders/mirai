use util::bytes::{BinaryWrite, MutableBuffer, VarInt};
use util::Result;
use util::Serialize;

use crate::network::ConnectedPacket;

/// Sent in response to [`ChunkRadiusRequest`](crate::ChunkRadiusRequest), to notify the client of the allowed render distance.
#[derive(Debug, Clone)]
pub struct ChunkRadiusReply {
    /// Maximum render distance that the server allows (in chunks).
    pub radius: i32,
}

impl ConnectedPacket for ChunkRadiusReply {
    const ID: u32 = 0x46;

    fn serialized_size(&self) -> usize {
        self.radius.var_len()
    }
}

impl Serialize for ChunkRadiusReply {
    fn serialize<W>(&self, buffer: W) -> anyhow::Result<()> where W: BinaryWrite {
        buffer.write_var_i32(self.radius)
    }
}
