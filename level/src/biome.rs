use std::mem;

use util::bytes::{BinaryRead, SharedBuffer};

use crate::ceil_div;

pub enum BiomeReturn {
    Empty,
    ReferBack,
    Data(Biome)
}

#[derive(Debug)]
pub struct Biome {
    heightmap: [[u16; 16]; 16]
}

impl Biome {
    pub fn deserialize<'a, R>(mut buffer: R) -> anyhow::Result<BiomeReturn> 
    where
        R: BinaryRead<'a>
    {
        const HM_BYTE_SIZE: usize = 512; // 16x16 u16 array
        
        let heightmap = if cfg!(target_endian = "little") {
            // SAFETY: This is safe because the data is made up of primitive integers only.
            // Additionally, this gives correct output because both the format and machine are little endian.
            unsafe {
                mem::transmute(buffer.take_const::<HM_BYTE_SIZE>()?)
            }
        } else {
            // If target not little endian, fall back to manually converting each
            // integer to the correct endianness.
            let mut heightmap = [[0u16; 16]; 16];
            for x in &mut heightmap {
                for y in x {
                    *y = buffer.read_u16_le()?;
                }
            }

            heightmap
        };

        loop {
            let array = Biome::deserialize_packed_array(&mut buffer)?;
        }

        todo!();
    }

    #[inline(always)]
    fn deserialize_packed_array<'a, R>(buffer: &mut R) -> anyhow::Result<Option<[u16; 4096]>> 
    where
        R: BinaryRead<'a>
    {
        let index_size = buffer.read_u8()?;
        if index_size == 0 {
            return Ok(None)
        }

        let per_word = ceil_div(u32::BITS, index_size as u32);
        let word_count = ceil_div(4096, per_word);

        let mut indices = [0u16; 4096];
        let mask = !(!0u32 << index_size);
        let mut offset = 0;

        for _ in 0..word_count {
            let mut word = buffer.read_u32_le()?;
            for _ in 0..per_word {
                indices[offset] = (word & mask) as u16;
                word >>= index_size;

                offset += 1;  
            }
        }

        Ok(Some(indices))
    }
}
