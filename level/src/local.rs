use util::{bail, Result};
use util::bytes::{BinaryRead, BinaryWrite, MutableBuffer};

use crate::{SubChunk, SubChunkVersion, SubLayer};

/// Performs ceiling division on two u32s.
#[inline]
const fn u32_ceil_div(lhs: u32, rhs: u32) -> u32 {
    (lhs + rhs - 1) / rhs
}

impl SubLayer {
    /// Deserializes a single layer from the given buffer.
    #[inline]
    fn deserialize_network<'a, R>(mut reader: R) -> anyhow::Result<Self>
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
        let word_count = 4096 / indices_per_word + match index_size {
            3 | 5 | 6 => 1,
            _ => 0
        };

        let mask = !(!0u32 << index_size);
        let mut indices = [0u16; 4096];
        for i in 0..word_count {
            let mut word = reader.read_u32_le()?;

            for j in 0..indices_per_word {
                let offset = i * indices_per_word + j;
                if offset == 4096 {
                    break
                }

                let index = word & mask;
                indices[i * indices_per_word + j] = index as u16;

                word >>= index_size;
            }
        }

        // Size of the block palette.
        let palette_size = reader.read_u32_le()?;
        let mut palette = Vec::with_capacity(palette_size as usize);
        for _ in 0..palette_size {
            let (entry, n) = nbt::from_le_bytes(reader)?;

            palette.push(entry);
            reader.advance(n)?;
        }

        Ok(Self { indices, palette })
    }

    fn serialize_network<W>(&self, mut writer: W) -> anyhow::Result<()>
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
            4096 / indices_per_word + padding
        };

        let mask = !(!0u32 << index_size);
        for i in 0..word_count {
            let mut word = 0;
            for j in 0..indices_per_word {
                let offset = i * indices_per_word + j;
                if offset == 4096 {
                    break
                }

                let index = self.indices[offset] as u32 & mask;

                word |= index;
                word <<= index_size;
            }

            writer.write_u32_le(word)?;
        }

        writer.write_u32_le(self.palette.len() as u32)?;
        for entry in &self.palette {
            nbt::to_le_bytes_in(&mut writer, entry)?;
        }

        Ok(())
    }
}

impl SubChunk {
    /// Deserialize a full sub chunk from the given buffer.
    pub fn deserialize_local<'a, R>(mut reader: R) -> anyhow::Result<Self>
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
            layers.push(SubLayer::deserialize_network(reader)?);
        }

        Ok(Self { version, index, layers })
    }

    /// Serialises the sub chunk into a new buffer and returns the buffer.
    ///
    /// Use [`serialize_local_in`](Self::serialize_local_in) to serialize into an existing writer.
    pub fn serialize_local(&self) -> anyhow::Result<MutableBuffer> {
        let mut buffer = MutableBuffer::new();
        self.serialize_local_in(&mut buffer)?;
        Ok(buffer)
    }

    /// Serialises the sub chunk into the given writer.
    pub fn serialize_local_in<W>(&self, mut writer: W) -> anyhow::Result<()>
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
            layer.serialize_network(&mut writer)?;
        }

        Ok(())
    }
}