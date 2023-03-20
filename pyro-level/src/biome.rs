use util::{Deserialize, Result};
use util::bytes::{BinaryReader, SharedBuffer};

#[derive(Debug)]
pub struct Biome3d {}

impl Biome3d {
    pub(crate) fn deserialize(mut buffer: SharedBuffer) -> Result<Self> {
        let index_size = buffer.read_u8()? >> 1;
        println!("is {index_size}");

        todo!();
    }
}
