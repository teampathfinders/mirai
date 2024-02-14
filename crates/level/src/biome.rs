use util::{BinaryRead, BinaryWrite};

use crate::PackedArrayReturn;

/// Size in bytes of the heightmap.
const HEIGHTMAP_SIZE: usize = 512;

/// A paletted biome.
///
/// This biome format is just like the sub chunk format.
/// Every block is an index into the palette, which is a list of biome IDs.
#[derive(Debug, PartialEq, Eq)]
pub struct PalettedBiome {
    /// Indices into the biome palette.
    pub(crate) indices: Box<[u16; 4096]>,
    /// Contains all biome IDs located in this chunk.
    pub(crate) palette: Vec<u32>,
}

impl PalettedBiome {
    /// A list of indices in this palette.
    #[inline]
    pub const fn indices(&self) -> &[u16; 4096] {
        &self.indices
    }

    /// Palette used by this biome.
    #[inline]
    pub fn palette(&self) -> &[u32] {
        &self.palette
    }
}

/// Represents the three different biome formats.
#[derive(Debug, PartialEq, Eq)]
pub enum BiomeEncoding {
    /// This sub chunk inherits all data from the previous sub chunk.
    Inherit,
    /// The entire sub chunk consists of a single biome.
    Single(u32),
    /// See [`PalettedBiome`].
    Paletted(PalettedBiome),
}

/// Describes the biomes contained in a single full size chunk.
///
/// The biome consists of a heightmap and a biome fragment for each sub chunk.
#[derive(Debug, PartialEq, Eq)]
pub struct Biomes {
    /// Highest blocks in the chunk.
    pub(crate) heightmap: Box<[[u16; 16]; 16]>,
    /// The biomes in each sub chunk.
    pub(crate) fragments: Vec<BiomeEncoding>,
}

impl Biomes {
    /// Heightmap of this biome.
    #[inline]
    pub const fn heightmap(&self) -> &[[u16; 16]; 16] {
        &self.heightmap
    }

    /// Fragments of this biome.
    #[inline]
    pub fn fragments(&self) -> &[BiomeEncoding] {
        &self.fragments
    }

    /// Reads a chunk biome from a raw buffer.
    pub(crate) fn deserialize<'a, R>(mut reader: R) -> anyhow::Result<Self>
    where
        R: BinaryRead<'a>,
    {
        let heightmap: Box<[[u16; 16]; 16]> = Box::new(bytemuck::cast(reader.take_const::<HEIGHTMAP_SIZE>()?));

        let mut fragments = Vec::new();
        while !reader.eof() {
            let packed_array = crate::deserialize_packed_array(&mut reader)?;
            if let PackedArrayReturn::Data(indices) = packed_array {
                let len = reader.read_u32_le()? as usize;

                let mut palette = Vec::with_capacity(len);
                for _ in 0..len {
                    palette.push(reader.read_u32_le()?);
                }

                fragments.push(BiomeEncoding::Paletted(PalettedBiome { indices, palette }));
            } else if PackedArrayReturn::Empty == packed_array {
                let single = reader.read_u32_le()?;
                fragments.push(BiomeEncoding::Single(single));
            } else {
                fragments.push(BiomeEncoding::Inherit);
            }
        }

        Ok(Self { heightmap, fragments })
    }

    /// Serializes the current chunk biome.
    pub(crate) fn serialize<W>(&self, mut writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite,
    {
        let cast = bytemuck::cast_slice(self.heightmap.as_ref());
        writer.write_all(cast)?;

        for fragment in &self.fragments {
            match fragment {
                BiomeEncoding::Inherit => writer.write_u8(0x7f << 1)?,
                BiomeEncoding::Single(single) => {
                    writer.write_u8(0)?;
                    writer.write_u32_le(*single)?;
                }
                BiomeEncoding::Paletted(biome) => {
                    crate::serialize_packed_array(&mut writer, &biome.indices, biome.palette.len(), false)?;

                    writer.write_u32_le(biome.palette.len() as u32)?;
                    for entry in &biome.palette {
                        writer.write_u32_le(*entry)?;
                    }
                }
            }
        }

        Ok(())
    }
}
