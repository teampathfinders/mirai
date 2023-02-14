use bytes::{Buf, BufMut, BytesMut};
use common::{bail, Decodable, Encodable, VResult};

const CHUNK_SIZE: usize = 4096;

#[derive(Debug, Clone)]
pub struct StorageRecord {
    indices: [u16; CHUNK_SIZE],
    palette: Vec<nbt::Value>,
}

impl StorageRecord {
    fn decode(buffer: &mut BytesMut) -> VResult<Self> {
        // Size of each index in bits.
        let index_size = buffer.get_u8() >> 1;
        if index_size == 0x7f {
            bail!(InvalidChunk, "Invalid block bit size {index_size}");
        }

        // Amount of indices that fit in a single 32-bit integer.
        let indices_per_word = u32::BITS as usize / index_size as usize;
        // Amount of words needed to encode 4096 block indices.
        let word_count = {
            let padding = match index_size {
                3 | 5 | 6 => 1,
                _ => 0,
            };
            CHUNK_SIZE / indices_per_word + padding
        };

        let mask = !(!0u32 << index_size);
        let mut indices = [0u16; CHUNK_SIZE];
        for i in 0..word_count {
            let mut word = buffer.get_u32_le();

            for j in 0..indices_per_word {
                let index = word & mask;
                indices[i * indices_per_word + j] = index as u16;

                word >>= index_size;
            }
        }

        // Size of the block palette.
        let palette_size = buffer.get_u32_le();

        let mut palette = Vec::with_capacity(palette_size as usize);
        for _ in 0..palette_size {
            let properties = nbt::read_le(buffer)?.value;
            palette.push(properties);
        }

        Ok(Self { indices, palette })
    }

    fn encode(&self, buffer: &mut BytesMut) {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct SubChunk {
    /// Version of the chunk.
    /// This version affects the format of the chunk.
    version: u8,
    index: u8,
    /// Layers of this chunk.
    /// The first layer contains blocks,
    /// the second layer contains waterlog data if it exists.
    storage_records: Vec<StorageRecord>,
}

impl Decodable for SubChunk {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        let version = buffer.get_u8();
        match version {
            1 => todo!(),
            8 | 9 => {
                let storage_count = buffer.get_u8();
                let index = if version == 9 { buffer.get_u8() } else { 0 };

                let mut storage_records =
                    Vec::with_capacity(storage_count as usize);

                for _ in 0..storage_count {
                    storage_records.push(StorageRecord::decode(&mut buffer)?);
                }

                Ok(Self { version, index, storage_records })
            }
            _ => bail!(InvalidChunk, "Invalid chunk version {version}"),
        }
    }
}

impl Encodable for SubChunk {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_u8(self.version);
        match self.version {
            1 => todo!(),
            8 | 9 => {
                buffer.put_u8(self.storage_records.len() as u8);

                if self.version == 9 {
                    buffer.put_u8(self.index);
                }

                for storage_record in &self.storage_records {
                    storage_record.encode(&mut buffer);
                }
            }
            _ => bail!(InvalidChunk, "Invalid chunk version {}", self.version),
        }

        Ok(buffer)
    }
}
