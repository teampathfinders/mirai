use util::{Deserialize, Result};
use util::{BinaryRead, SharedBuffer};

use crate::bedrock::ConnectedPacket;

#[derive(Debug, Clone)]
pub struct CacheBlobStatus {
    /// Hashes of the blobs that the client still needs.
    pub misses: Vec<u64>,
    /// Hashes of the blobs that the client already possesses.
    /// These do not have to be sent by the server.
    pub hits: Vec<u64>,
}

impl ConnectedPacket for CacheBlobStatus {
    const ID: u32 = 0x87;
}

impl<'a> Deserialize<'a> for CacheBlobStatus {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let miss_count = reader.read_var_u32()?;
        let hit_count = reader.read_var_u32()?;

        let mut misses = Vec::with_capacity(miss_count as usize);
        for _ in 0..miss_count {
            misses.push(reader.read_u64_le()?);
        }

        let mut hits = Vec::with_capacity(hit_count as usize);
        for _ in 0..hit_count {
            hits.push(reader.read_u64_le()?);
        }

        Ok(Self {
            misses,
            hits,
        })
    }
}

