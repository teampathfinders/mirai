use bytes::Bytes;
use util::{Result};
use util::bytes::SharedBuffer;

use util::Deserialize;
use crate::network::packets::ConnectedPacket;

/// Sent by the client to request the maximum render distance.
#[derive(Debug)]
pub struct ChunkRadiusRequest {
    /// Requested render distance (in chunks).
    pub radius: i32,
}

impl ConnectedPacket for ChunkRadiusRequest {
    const ID: u32 = 0x45;
}

impl Deserialize for ChunkRadiusRequest {
    fn deserialize(mut buffer: SharedBuffer) -> Result<Self> {
        let radius = buffer.read_var::<i32>()?;

        Ok(Self { radius })
    }
}
