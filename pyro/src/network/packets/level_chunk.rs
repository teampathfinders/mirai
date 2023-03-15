use bytes::{BufMut, BytesMut, Bytes};
use util::{Result, Vector2i, WriteExtensions};

use util::Serialize;

use super::ConnectedPacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SubChunkRequestMode {
    /// The legacy mode that specifies the amount of sub chunks in the packet.
    Legacy,
    /// Limitless mode that allows an unlimited world height.
    Limitless,
    /// Mode that only specifies the highest chunk.
    Limited,
}

#[derive(Debug, Clone)]
pub struct LevelChunk {
    /// Position of the chunk.
    pub position: Vector2i,
    /// How these chunks should be handled by the client.
    pub request_mode: SubChunkRequestMode,
    /// Top sub chunk in the packet.
    /// This is used if the request mode is set to limited.
    pub highest_sub_chunk: u16,
    /// Amount of sub chunks in this packet.
    pub sub_chunk_count: u32,
    /// List of hashes used to cache the chunks.
    /// This should be set to None if the client does not support the blob cache.
    pub blob_hashes: Option<Vec<u64>>,
    /// Raw chunk data.
    pub raw_payload: BytesMut,
}

impl ConnectedPacket for LevelChunk {
    const ID: u32 = 0x3a;
}

impl Serialize for LevelChunk {
    fn serialize(&self, buffer: &mut BytesMut) {
        let mut buffer = BytesMut::new();

        buffer.put_vec2i(&self.position);
        match self.request_mode {
            SubChunkRequestMode::Legacy => {
                buffer.put_var_u32(self.sub_chunk_count);
            }
            SubChunkRequestMode::Limitless => {
                buffer.put_var_u32(u32::MAX);
            }
            SubChunkRequestMode::Limited => {
                buffer.put_var_u32(u32::MAX - 1);
                buffer.write_be::<u16>()(self.highest_sub_chunk);
            }
        }

        buffer.write_bool(self.blob_hashes.is_some());
        if let Some(hashes) = &self.blob_hashes {
            buffer.put_var_u32(hashes.len() as u32);
            for hash in hashes {
                buffer.write_be::<u64>()(*hash);
            }
        }

        buffer.put(self.raw_payload.as_ref());
    }
}
