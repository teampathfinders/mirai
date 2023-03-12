use bytes::{Buf, Bytes};
use util::{Deserialize, Result};
use util::bytes::ReadBuffer;

#[derive(Debug)]
pub struct Biome3d {}

impl Deserialize for Biome3d {
    fn deserialize(mut buffer: ReadBuffer) -> Result<Self> {
        let index_size = buffer.read_le::<u8>()? >> 1;
        println!("is {index_size}");

        todo!();
    }
}
