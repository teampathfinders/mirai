use util::{BinaryWrite, RVec, Serialize, Vector};

use crate::{bedrock::ConnectedPacket, types::Dimension};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SubChunkRequestMode {
    /// The legacy mode that specifies the amount of sub chunks in the packet.
    Legacy { subchunk_count: u32 },
    /// Limitless mode that allows an unlimited world height.
    Request,
    /// Mode that only specifies the highest non-air subchunk relative to the bottom of the level chunk.
    KnownAir { highest_nonair: u16 },
}

#[derive(Debug)]
pub struct LevelChunk {
    /// Position of the chunk.
    pub coordinates: Vector<i32, 2>,
    /// Dimension of the chunk.
    pub dimension: Dimension,
    /// How these chunks should be handled by the client.
    pub request_mode: SubChunkRequestMode,
    /// List of hashes used to cache the chunks.
    /// This should be set to None if the client does not support the blob cache.
    pub blob_hashes: Option<Vec<u64>>,
    /// Raw chunk data.
    pub raw_payload: RVec,
}

impl ConnectedPacket for LevelChunk {
    const ID: u32 = 0x3a;
}

impl Serialize for LevelChunk {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_veci(&self.coordinates)?;
        writer.write_var_i32(self.dimension as i32)?;
        match self.request_mode {
            SubChunkRequestMode::Legacy { subchunk_count } => {
                writer.write_var_u32(subchunk_count)?;
            }
            SubChunkRequestMode::Request => {
                writer.write_var_u32(u32::MAX)?;
            }
            SubChunkRequestMode::KnownAir { highest_nonair: highest_subchunk } => {
                writer.write_var_u32(u32::MAX - 1)?;
                writer.write_u16_le(highest_subchunk)?;
            }
        }

        writer.write_bool(self.blob_hashes.is_some())?;
        if let Some(hashes) = &self.blob_hashes {
            writer.write_var_u32(hashes.len() as u32)?;
            for hash in hashes {
                writer.write_u64_le(*hash)?;
            }
        }

        writer.write_var_u32(self.raw_payload.len() as u32)?;
        writer.write_all(self.raw_payload.as_ref())?;
        Ok(())
    }
}
