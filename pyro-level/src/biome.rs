use bytes::{Buf, Bytes};
use util::bytes::{BinaryBuffer, SharedBuffer};
use util::{Deserialize, Result};

#[derive(Debug)]
pub struct Biome3d {}

impl Biome3d {
    pub(crate) fn deserialize(mut buffer: SharedBuffer) -> Result<Self> {
        let index_size = buffer.read_le::<u8>()? >> 1;
        println!("is {index_size}");

        todo!();
    }
}
