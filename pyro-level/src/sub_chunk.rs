use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use util::{bail, Error, Result, Vector};
use util::bytes::{BinaryRead, BinaryWrite, MutableBuffer};

const CHUNK_SIZE: usize = 4096;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SubChunkVersion {
    Legacy = 1,
    Limited = 8,
    Limitless = 9,
}

impl TryFrom<u8> for SubChunkVersion {
    type Error = Error;

    fn try_from(v: u8) -> Result<Self> {
        Ok(match v {
            1 => Self::Legacy,
            8 => Self::Limited,
            9 => Self::Limitless,
            _ => bail!(Malformed, "Invalid chunk version: {v}"),
        })
    }
}

/// Performs ceiling division on two u32s.
#[inline]
fn u32_ceil_div(lhs: u32, rhs: u32) -> u32 {
    (lhs + rhs - 1) / rhs
}

mod block_version {
    use serde::{Deserialize, Deserializer, Serializer};

    #[inline]
    pub fn deserialize<'de, D>(de: D) -> Result<Option<[u8; 4]>, D::Error>
        where
            D: Deserializer<'de>
    {
        let word = Option::<i32>::deserialize(de)?;
        Ok(word.map(|w| w.to_be_bytes()))
    }

    #[inline]
    pub fn serialize<S>(v: &Option<[u8; 4]>, ser: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer
    {
        if let Some(b) = v {
            ser.serialize_i32(i32::from_be_bytes(*b))
        } else {
            ser.serialize_none()
        }
    }
}

/// Definition of block in the sub chunk block palette.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct PaletteEntry {
    /// Name of the block.
    pub name: String,
    /// Version of the block.
    #[serde(with = "block_version")]
    pub version: Option<[u8; 4]>,
    /// Block-specific properties.
    pub states: Option<HashMap<String, nbt::Value>>,
}

/// A layer in a sub chunk.
///
/// Sub chunks can have multiple layers.
/// The first layer contains plain old block data,
/// while the second layer (if it exists) generally contains water logging data.
///
/// The layer is prefixed with a byte indicating the size in bits of the block indices.
/// This is followed by `4096 / (32 / bits)` 32-bit integers containing the actual indices.
/// In case the size is 3, 5 or 6, there is one more integer appended to the end to fit all data.
///
/// Immediately following the indices, the palette starts.
/// This is prefixed with a 32-bit little endian integer specifying the size of the palette.
/// The rest of the palette then consists of `n` concatenated NBT compounds.
#[doc(alias = "storage record")]
#[derive(Debug)]
pub struct SubLayer {
    /// List of indices into the palette.
    ///
    /// Coordinates can be converted to an offset into the array using [`to_offset`].
    indices: [u16; CHUNK_SIZE],
    /// List of all different block types in this sub chunk layer.
    palette: Vec<PaletteEntry>,
}

impl SubLayer {
    /// Creates an iterator over the blocks in this layer.
    ///
    /// This iterates over every indices
    #[inline]
    pub fn iter(&self) -> LayerIter {
        LayerIter::from(self)
    }

    /// Deserializes a single layer from the given buffer.
    #[inline]
    fn deserialize<'a, R>(mut reader: R) -> Result<Self>
    where
        R: BinaryRead<'a> + Copy + 'a
    {
        // Size of each index in bits.
        let index_size = reader.read_u8()? >> 1;
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
            let mut word = reader.read_u32_le()?;

            for j in 0..indices_per_word {
                let index = word & mask;
                indices[i * indices_per_word + j] = index as u16;

                word >>= index_size;
            }
        }

        // Padded sizes have an extra word.
        match index_size {
            3 | 5 | 6 => {
                let mut word = reader.read_u32_le()?;
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
        let palette_size = reader.read_u32_le()?;
        let mut palette = Vec::with_capacity(palette_size as usize);
        // let mut palette = Vec::new();
        for _ in 0..palette_size {
            let (entry, n) = nbt::from_le_bytes(reader)?;

            palette.push(entry);
            reader.advance(n)?;
        }

        Ok(Self { indices, palette })
    }

    fn serialize<W>(&self, mut writer: W) -> Result<()>
    where
        W: BinaryWrite
    {
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

        writer.write_u8(index_size << 1)?;

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

            writer.write_u32_le(word)?;
        }

        writer.write_u32_le(self.palette.len() as u32)?;
        for entry in &self.palette {
            todo!("serialize BlockProperties nbt");
            // nbt::serialize_le("", entry, buffer);
        }

        Ok(())
    }
}

impl Default for SubLayer {
    // Std using const generics for arrays would be really nice...
    fn default() -> Self {
        Self { indices: [0; 4096], palette: Vec::new() }
    }
}

/// Converts coordinates to offsets into the block palette indices.
///
/// These coordinates should be in the range [0, 16) for each component.
#[inline]
pub fn to_offset(position: Vector<u8, 3>) -> usize {
    16 * 16 * position.x as usize
        + 16 * position.z as usize
        + position.y as usize
}

/// Converts an offset back to coordinates.
///
/// This offset should be in the range [0, 4096).
#[inline]
pub fn from_offset(offset: usize) -> Vector<u8, 3> {
    Vector::from([
        (offset >> 8) as u8 & 0xf,
        offset as u8 & 0xf,
        (offset >> 4) as u8 & 0xf,
    ])
}

/// A Minecraft sub chunk.
///
/// Every world contains
#[derive(Debug)]
pub struct SubChunk {
    /// Version of the sub chunk.
    ///
    /// See [`SubChunkVersion`] for more info.
    version: SubChunkVersion,
    /// Index of the sub chunk.
    ///
    /// This specifies the vertical position of the sub chunk.
    /// It is only used if `version` is set to [`Limitless`](SubChunkVersion::Limitless)
    /// and set to 0 otherwise.
    index: i8,
    /// Layers the sub chunk consists of.
    ///
    /// See [`SubLayer`] for more info.
    ///
    // Surprisingly using a Vec is faster than using a SmallVec.
    // This is probably because of the expensive copy of `SubLayer::indices`
    layers: Vec<SubLayer>,
}

impl SubChunk {
    /// Gets a block at the specified position.
    ///
    /// See [`to_offset`] for the requirements on `position`.
    pub fn get(&self, position: Vector<u8, 3>) -> Option<&PaletteEntry> {
        let offset = to_offset(position);
        let index = self.layers[0].indices[offset];

        self.layers[0].palette.get(index as usize)
    }

    /// Returns the specified layer from the sub chunk.
    #[inline]
    pub fn layer(&self, index: usize) -> Option<&SubLayer> {
        self.layers.get(index)
    }
}

impl SubChunk {
    /// Deserialize a full sub chunk from the given buffer.
    pub fn deserialize<'a, R>(mut reader: R) -> Result<Self>
    where
        R: BinaryRead<'a> + Copy + 'a,
    {
        let version = SubChunkVersion::try_from(reader.read_u8()?)?;
        let layer_count = match version {
            SubChunkVersion::Legacy => 1,
            _ => reader.read_u8()?,
        };

        if layer_count == 0 || layer_count > 2 {
            bail!(Malformed, "Sub chunk must have 1 or 2 layers");
        }

        let index = if version == SubChunkVersion::Limitless {
            reader.read_i8()?
        } else {
            0
        };

        // let mut layers = SmallVec::with_capacity(layer_count as usize);
        let mut layers = Vec::with_capacity(layer_count as usize);
        for _ in 0..layer_count {
            layers.push(SubLayer::deserialize(reader)?);
        }

        Ok(Self { version, index, layers })
    }

    /// Serialises the sub chunk into a new buffer and returns the buffer.
    ///
    /// Use [`serialize_in`](Self::serialize_in) to serialize into an existing writer.
    pub fn serialize(&self) -> Result<MutableBuffer> {
        let mut buffer = MutableBuffer::new();
        self.serialize_in(&mut buffer)?;
        Ok(buffer)
    }

    /// Serialises the sub chunk into the given writer.
    pub fn serialize_in<W>(&self, mut writer: W) -> Result<()>
    where
        W: BinaryWrite
    {
        writer.write_u8(self.version as u8)?;
        match self.version {
            SubChunkVersion::Legacy => writer.write_u8(1),
            _ => writer.write_u8(self.layers.len() as u8)
        }?;

        if self.version == SubChunkVersion::Limitless {
            writer.write_i8(self.index)?;
        }

        for layer in &self.layers {
            layer.serialize(&mut writer)?;
        }

        Ok(())
    }
}

/// Iterator over blocks in a layer.
pub struct LayerIter<'a> {
    /// Indices in the sub chunk.
    /// While iterating, this is slowly consumed by `std::slice::split_at`.
    indices: &'a [u16],
    /// All possible block states in the current chunk.
    palette: &'a [PaletteEntry],
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
    type Item = &'a PaletteEntry;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // ExactSizeIterator::is_empty is unstable
        if self.len() == 0 {
            return None;
        }

        let (a, b) = self.indices.split_at(1);
        self.indices = b;
        self.palette.get(a[0] as usize)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.len()))
    }
}

impl<'a> ExactSizeIterator for LayerIter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.indices.len()
    }
}
