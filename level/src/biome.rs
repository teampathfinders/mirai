use util::bytes::{BinaryRead, SharedBuffer};

#[derive(Debug)]
pub struct Biome {

}

impl Biome {
    pub(crate) fn deserialize(mut buffer: SharedBuffer) -> anyhow::Result<Self> {
        let index_size = buffer.read_u8()? >> 1;
        println!("is {index_size}");

        let palette_size = buffer.read_u32_le()?;
        dbg!(palette_size);

        todo!();
    }
}
