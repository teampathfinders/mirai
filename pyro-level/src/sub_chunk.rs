use bytes::{Buf, BufMut, Bytes, BytesMut};
use util::{bail, BlockPosition, Error, Result, Vector3b};
use nbt::ReadBuffer;
use serde::Deserialize;
use std::collections::HashMap;
use std::iter::Enumerate;
use util::bytes::ReadBuffer;

const CHUNK_SIZE: usize = 4096;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SubChunkVersion {
    Legacy = 1,
    Limited = 8,
    Limitless = 9,
}

#[inline]
fn u32_ceil_div(lhs: u32, rhs: u32) -> u32 {
    (lhs + rhs - 1) / rhs
}

mod block_version {
    use serde::{Deserialize, Serialize};
    use util::bytes::FromBytes;

    pub fn deserialize<'de, D>(d: D) -> Result<[u8; 4], D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let int = i32::deserialize(d)?;
        Ok(int.to_be_bytes())
    }

    pub fn serialize<S>(v: [u8; 4], s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let int = i32::from_be_bytes(v);
        int.serialize(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct BlockStates {
    pub pillar_axis: Option<String>,
    pub dirt_type: Option<String>,
    pub stone_type: Option<String>,
    pub double_plant_type: Option<String>,
    #[serde(default)]
    pub upper_block_bit: bool,
    pub dripstone_thickness: Option<String>,
    #[serde(default)]
    pub hanging: bool, // pub liquid_depth: Option<i8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct BlockProperties {
    pub name: String,
    #[serde(with = "block_version")]
    pub version: [u8; 4],
    pub states: BlockStates,
}

#[derive(Debug, Clone)]
pub struct SubLayer {
    indices: [u16; CHUNK_SIZE],
    palette: Vec<BlockProperties>,
}

impl SubLayer {
    #[inline]
    pub fn iter(&self) -> LayerIter {
        LayerIter::from(self)
    }

    fn deserialize(buffer: &mut ReadBuffer) -> Result<Self> {
        // Size of each index in bits.
        let index_size = buffer.read_le::<u8>()? >> 1;
        if index_size == 0x7f {
            bail!(Malformed, "Invalid block bit size {index_size}");
        }

        // Amount of indices that fit in a single 32-bit integer.
        let indices_per_word = u32::BITS as usize / index_size as usize;
        // Amount of words needed to encode 4096 block indices.
        let word_count = CHUNK_SIZE / indices_per_word;

        let mask = !(!0u32 << index_size);
        let mut indices = [0u16; CHUNK_SIZE];
        for i in 0..word_count {
            // println!("{i} {}", i * indices_per_word);
            let mut word = buffer.read_le::<u32>()?;

            for j in 0..indices_per_word {
                let index = word & mask;
                indices[i * indices_per_word + j] = index as u16;

                word >>= index_size;
            }
        }

        // Padded sizes have an extra word.
        match index_size {
            3 | 5 | 6 => {
                let mut word = buffer.read_le::<u32>()?;
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
        let palette_size = buffer.read_le::<u32>()?;
        let mut palette = Vec::with_capacity(palette_size as usize);
        for _ in 0..palette_size {
            let (properties, n) = match nbt::from_le_bytes(buffer) {
                Ok(p) => p,
                Err(e) => {
                    bail!(Malformed, "{}", e.to_string())
                }
            };

            palette.push(properties);
            buffer.advance(n + 3);
        }

        Ok(Self { indices, palette })
    }

    fn serialize(&self, buffer: &mut WriteBuffer) {
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
            todo!("serialize BlockProperties nbt");
            // nbt::serialize_le("", entry, buffer);
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
    version: SubChunkVersion,
    /// Chunk index.
    index: i8,
    /// Layers of this chunk.
    /// The first layer contains blocks,
    /// the second layer contains waterlog data if it exists.
    layers: Vec<SubLayer>,
}

impl SubChunk {
    pub fn get(&self, position: Vector3b) -> Option<&BlockProperties> {
        let offset = pos_to_offset(position);
        let index = self.layers[0].indices[offset];

        self.layers[0].palette.get(index as usize)
    }

    #[inline]
    pub fn layer(&self, index: usize) -> Option<&SubLayer> {
        self.layers.get(index)
    }
}

impl util::Deserialize for SubChunk {
    fn deserialize(mut buffer: ReadBuffer) -> Result<Self> {
        let version = buffer.read_le::<u8>()?;
        match version {
            1 => todo!(),
            8 | 9 => {
                let storage_count = buffer.read_le::<u8>()?;
                let index = if version == 9 {
                    buffer.read_le::<i8>()?
                } else {
                    0
                };

                let mut storage_records =
                    Vec::with_capacity(storage_count as usize);

                for _ in 0..storage_count {
                    storage_records.push(SubLayer::deserialize(&mut buffer)?);
                }

                let version = if version == 8 {
                    SubChunkVersion::Limited
                } else {
                    SubChunkVersion::Limitless
                };

                Ok(Self { version, index, layers: storage_records })
            }
            _ => bail!(Malformed, "Invalid chunk version {version}"),
        }
    }
}

impl util::Serialize for SubChunk {
    fn serialize(&self, buffer: &mut WriteBuffer) {
        buffer.put_u8(self.version as u8);
        match self.version {
            SubChunkVersion::Legacy => todo!(),
            _ => {
                buffer.put_u8(self.layers.len() as u8);

                if self.version == SubChunkVersion::Limitless {
                    buffer.put_i8(self.index);
                }

                for storage_record in &self.layers {
                    storage_record.serialize(buffer);
                }
            }
        }
    }
}

pub struct LayerIter<'a> {
    indices: &'a [u16],
    palette: &'a [BlockProperties],
}

impl<'a> From<&'a SubLayer> for LayerIter<'a> {
    #[inline]
    fn from(value: &'a SubLayer) -> Self {
        Self {
            indices: &value.indices,
            palette: value.palette.as_ref(),
        }
    }
}

impl<'a> Iterator for LayerIter<'a> {
    type Item = &'a BlockProperties;

    fn next(&mut self) -> Option<Self::Item> {
        // ExactSizeIterator::is_empty is unstable
        if self.len() == 0 {
            return None;
        }

        let (a, b) = self.indices.split_at(1);
        self.indices = b;
        self.palette.get(a[0] as usize)
    }
}

impl<'a> ExactSizeIterator for LayerIter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.indices.len()
    }
}
