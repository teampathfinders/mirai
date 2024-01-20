use util::{BinaryRead};
use util::Deserialize;
use util::Result;

use crate::bedrock::ConnectedPacket;

/// Sent by the client to request the maximum render distance.
#[derive(Debug)]
pub struct ChunkRadiusRequest {
    /// Requested render distance (in chunks).
    pub radius: i32,
}

impl ConnectedPacket for ChunkRadiusRequest {
    const ID: u32 = 0x45;
}

impl<'a> Deserialize<'a> for ChunkRadiusRequest {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let radius = reader.read_var_i32()?;
        let max_radius = reader.read_u8()?;

        Ok(Self { radius })
    }
}
