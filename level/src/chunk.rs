use bytes::{Buf, BytesMut};
use common::{bail, Decodable, VResult};

const CHUNK_SIZE: usize = 4096;

#[derive(Debug)]
pub struct StorageRecord {
    indices: [u16; CHUNK_SIZE],
}

impl StorageRecord {
    pub fn decode(buffer: &mut BytesMut) -> VResult<Self> {
        // Size of each index in bits.
        let index_size = buffer.get_u8() >> 1;
        if index_size == 0x7f {
            bail!(InvalidChunk, "Invalid block bit size {bits_per_block}");
        }

        // Amount of indices that fit in a single 32-bit integer.
        let indices_per_word = 32 / index_size as usize;
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

        // let mut palette = Vec::with_capacity(palette_size as usize);
        for _ in 0..palette_size {
            let properties = nbt::read_le(buffer)?;
            println!("{properties:?}");
        }

        Ok(Self { indices })
    }
}

#[derive(Debug)]
pub struct SubChunk {
    /// Version of the chunk.
    /// This version affects the format of the chunk.
    version: u8,
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

                if version == 9 {
                    buffer.advance(1); // Skip chunk index
                }

                let mut storage_records = Vec::with_capacity(storage_count as usize);
                for _ in 0..storage_count {
                    storage_records.push(StorageRecord::decode(&mut buffer)?);
                }

                Ok(Self {
                    version,
                    storage_records,
                })
            }
            _ => bail!(InvalidChunk, "Invalid chunk version {chunk_version}"),
        }
    }
}