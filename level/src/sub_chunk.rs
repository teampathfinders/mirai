use bytes::{Buf, BufMut, Bytes, BytesMut};
use common::{bail, BlockPosition, Deserialize, Serialize, VResult, Vector3b};

const CHUNK_SIZE: usize = 4096;

#[inline]
fn u32_ceil_div(lhs: u32, rhs: u32) -> u32 {
    (lhs + rhs - 1) / rhs
}

#[derive(Debug, Clone)]
pub struct StorageRecord {
    indices: [u16; CHUNK_SIZE],
    palette: Vec<nbt::Value>,
}

impl StorageRecord {
    fn deserialize(buffer: &mut Bytes) -> VResult<Self> {
        // Size of each index in bits.
        let index_size = buffer.get_u8() >> 1;
        if index_size == 0x7f {
            bail!(InvalidChunk, "Invalid block bit size {index_size}");
        }

        // Amount of indices that fit in a single 32-bit integer.
        let indices_per_word = u32::BITS as usize / index_size as usize;
        // Amount of words needed to encode 4096 block indices.
        let word_count = CHUNK_SIZE / indices_per_word;

        let mask = !(!0u32 << index_size);
        let mut indices = [0u16; CHUNK_SIZE];
        for i in 0..word_count {
            // println!("{i} {}", i * indices_per_word);
            let mut word = buffer.get_u32_le();

            for j in 0..indices_per_word {
                let index = word & mask;
                indices[i * indices_per_word + j] = index as u16;

                word >>= index_size;
            }
        }

        // Padded sizes have an extra word.
        match index_size {
            3 | 5 | 6 => {
                let mut word = buffer.get_u32_le();
                let last_index =
                    (word_count - 1) * indices_per_word + indices_per_word - 1;

                let indices_left = 4096 - last_index;
                for i in 0..indices_left {
                    indices[last_index + i] = (word & mask) as u16;
                    word >>= index_size;
                }
            }
            _ => (),
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

    fn serialize(&self, buffer: &mut BytesMut) {
        // Determine the required bits per index
        let index_size = {
            let palette_size = self.palette.len();

            let mut bits_per_block = 0;
            // Loop over allowed values.
            for b in [1, 2, 3, 4, 5, 6, 8, 16] {
                if 2usize.pow(b) >= palette_size {
                    bits_per_block = b;
                    break;
                }
            }

            bits_per_block as u8
        };

        buffer.put_u8(index_size << 1);

        // Amount of indices that fit in a single 32-bit integer.
        let indices_per_word =
            u32_ceil_div(u32::BITS, index_size as u32) as usize;
        // Amount of words needed to encode 4096 block indices.
        let word_count = {
            let padding = match index_size {
                3 | 5 | 6 => 1,
                _ => 0,
            };
            CHUNK_SIZE / indices_per_word + padding
        };

        let mask = !(!0u32 << index_size);
        for i in 0..word_count {
            let mut word = 0;
            for j in 0..indices_per_word {
                let index =
                    self.indices[i * indices_per_word + j] as u32 & mask;
                word |= index;
                word <<= indices_per_word;
            }

            buffer.put_u32_le(word);
        }

        buffer.put_u32_le(self.palette.len() as u32);
        for entry in &self.palette {
            nbt::write_le("", entry, buffer);
        }
    }
}

fn pos_to_offset(position: Vector3b) -> usize {
    16 * 16 * position.x as usize
        + 16 * position.z as usize
        + position.y as usize
}

fn offset_to_pos(offset: usize) -> Vector3b {
    Vector3b::from([
        (offset >> 8) as u8 & 0xf,
        (offset >> 0) as u8 & 0xf,
        (offset >> 4) as u8 & 0xf,
    ])
}

/// Represents the blocks in a sub chunk.
#[derive(Debug, Clone)]
pub struct SubChunk {
    /// Version of the chunk.
    /// This version affects the format of the chunk.
    version: u8,
    /// Chunk index.
    index: u8,
    /// Layers of this chunk.
    /// The first layer contains blocks,
    /// the second layer contains waterlog data if it exists.
    storage_records: Vec<StorageRecord>,
}

impl SubChunk {
    pub fn get(&self, position: Vector3b) -> Option<&nbt::Value> {
        let offset = pos_to_offset(position);
        let index = self.storage_records[0].indices[offset];

        self.storage_records[0].palette.get(index as usize)
    }
}

impl Deserialize for SubChunk {
    fn deserialize(mut buffer: Bytes) -> VResult<Self> {
        let version = buffer.get_u8();
        match version {
            1 => todo!(),
            8 | 9 => {
                let storage_count = buffer.get_u8();
                let index = if version == 9 { buffer.get_u8() } else { 0 };

                let mut storage_records =
                    Vec::with_capacity(storage_count as usize);

                println!("Decoding {storage_count} records");
                for _ in 0..storage_count {
                    storage_records
                        .push(StorageRecord::deserialize(&mut buffer)?);
                }

                Ok(Self { version, index, storage_records })
            }
            _ => bail!(InvalidChunk, "Invalid chunk version {version}"),
        }
    }
}

impl Serialize for SubChunk {
    fn serialize(&self, buffer: &mut BytesMut) {
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
                    storage_record.serialize(&mut buffer);
                }
            }
            _ => bail!(InvalidChunk, "Invalid chunk version {}", self.version),
        }

        Ok(buffer.freeze())
    }
}
