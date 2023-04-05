use std::mem;

use util::bytes::{BinaryRead, SharedBuffer};

use crate::{ceil_div, PackedArrayReturn};

#[derive(Debug)]
pub struct PalettedBiome {
    indices: Box<[u16; 4096]>,
    palette: Vec<u32>,
}

#[derive(Debug)]
pub enum BiomeFragment {
    Referral(u8),
    Single(u32),
    Paletted(PalettedBiome),
}

#[derive(Debug)]
pub struct Biome {
    heightmap: Box<[[u16; 16]; 16]>,
    fragments: Vec<BiomeFragment>,
}

impl Biome {
    pub(crate) fn deserialize<'a, R>(mut reader: R) -> anyhow::Result<Self>
    where
        R: BinaryRead<'a>,
    {
        const HM_BYTE_SIZE: usize = 512; // 16x16 u16 array

        let heightmap: Box<[[u16; 16]; 16]> = Box::new(bytemuck::cast(reader.take_const::<HM_BYTE_SIZE>()?));

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
                fragments.push(BiomeFragment::Referral(fragments.len() as u8 - 1));
            }
        }

        Ok(Self { heightmap, fragments })
    }
}
