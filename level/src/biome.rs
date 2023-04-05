use std::mem;

use util::bytes::{BinaryRead, BinaryWrite, SharedBuffer};

use crate::{ceil_div, PackedArrayReturn};

const HEIGHTMAP_SIZE: usize = 512; // 16x16 u16 array

#[derive(Debug, PartialEq, Eq)]
pub struct PalettedBiome {
    pub(crate) indices: Box<[u16; 4096]>,
    pub(crate) palette: Vec<u32>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BiomeFragment {
    Referral,
    Single(u32),
    Paletted(PalettedBiome),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Biome {
    pub(crate) heightmap: Box<[[u16; 16]; 16]>,
    pub(crate) fragments: Vec<BiomeFragment>,
}

impl Biome {
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

                fragments.push(BiomeFragment::Paletted(PalettedBiome { indices, palette }));
            } else if PackedArrayReturn::Empty == packed_array {
                let single = reader.read_u32_le()?;
                fragments.push(BiomeFragment::Single(single));
            } else {
                fragments.push(BiomeFragment::Referral);
            }
        }

        Ok(Self { heightmap, fragments })
    }

    pub(crate) fn serialize<W>(&self, mut writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite,
    {
        let cast = bytemuck::cast_slice(self.heightmap.as_ref());
        writer.write_all(cast)?;

        for fragment in &self.fragments {
            match fragment {
                BiomeFragment::Referral => writer.write_u8(0x7f << 1)?,
                BiomeFragment::Single(single) => {
                    writer.write_u8(0)?;
                    writer.write_u32_le(*single)?;
                }
                BiomeFragment::Paletted(biome) => {
                    crate::serialize_packed_array(&mut writer, &biome.indices, biome.palette.len())?;

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
