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
    pub fn deserialize<'a, R>(mut reader: R) -> anyhow::Result<BiomeReturn> 
    where
        R: BinaryRead<'a>
    {
        const HM_BYTE_SIZE: usize = 512; // 16x16 u16 array
    
        let heightmap: [[u16; 16]; 16] = bytemuck::cast(reader.take_const::<HM_BYTE_SIZE>()?);

        let mut y_index = 0;
        loop {
            let array = crate::deserialize_packed_array(&mut reader)?;
            if let Some(indices) = array {
                anyhow::bail!("Index size was 0");
            } else {
                let len = reader.read_u32_le()? * 4;
            
                println!("{:?}", reader.take_n(len as usize)?);

                // let indices = reader
                //     .take_n(len as usize)?
                //     .windows(4)
                //     .map(|w| u32::from_be_bytes(w.try_into().unwrap()))
                //     .collect::<Vec<u32>>();
                

                // println!("v{indices:?}");

                break
            }
            
            y_index += 1;
        }

        todo!();
    }
}
