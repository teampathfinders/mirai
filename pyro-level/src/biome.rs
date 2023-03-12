use bytes::{Buf, Bytes};
use common::{Deserialize, Result};
#[derive(Debug)]
pub struct Biome3d {}

impl Deserialize for Biome3d {
    fn deserialize(mut buffer: Bytes) -> Result<Self> {
        let index_size = buffer.get_u8() >> 1;
        println!("is {index_size}");

        todo!();
    }
}
