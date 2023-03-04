use bytes::{Bytes, Buf};
use common::{Deserialize, VResult};
#[derive(Debug)]
pub struct Biome3d {

}

impl Deserialize for Biome3d {
    fn deserialize(mut buffer: Bytes) -> VResult<Self> {
        let index_size = buffer.get_u8() >> 1;
        println!("is {index_size}");

        

        todo!();
    }
}