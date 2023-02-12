use common::{ReadExtensions, VResult};

use crate::network::Decodable;

use super::GamePacket;

/// Sent by the client to request the maximum render distance.
#[derive(Debug)]
pub struct ChunkRadiusRequest {
    /// Requested render distance (in chunks).
    pub radius: i32,
}

impl GamePacket for ChunkRadiusRequest {
    const ID: u32 = 0x45;
}

impl Decodable for ChunkRadiusRequest {
    fn decode(mut buffer: bytes::BytesMut) -> VResult<Self> {
        let radius = buffer.get_var_i32()?;

        Ok(Self { radius })
    }
}
