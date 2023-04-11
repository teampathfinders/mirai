use util::bytes::{BinaryRead, SharedBuffer};
use util::Deserialize;
use util::Result;

use crate::network::ConnectedPacket;

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
    fn deserialize<R>(reader: R) -> anyhow::Result<Self>
    where
        R: BinaryRead<'a> + 'a
    {
        let radius = reader.read_var_i32()?;

        Ok(Self { radius })
    }
}
