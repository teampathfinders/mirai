use util::bytes::{BinaryRead, SharedBuffer};

#[derive(Debug)]
pub struct Biome {

}

impl Biome {
    pub fn deserialize<'a, R>(mut buffer: R) -> anyhow::Result<Self> 
    where
        R: BinaryRead<'a>
    {
        let index_size = buffer.read_u8()? >> 1;
        println!("is {index_size}");

        

        let palette_size = buffer.read_u32_le()?;
        dbg!(palette_size);

        todo!();
    }
}
