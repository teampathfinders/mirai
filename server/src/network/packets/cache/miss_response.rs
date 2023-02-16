use bytes::BytesMut;
use common::{Encodable, VResult, WriteExtensions};
use crate::network::packets::GamePacket;

#[derive(Debug, Clone)]
pub struct CacheMissResponse<'a> {
    pub blobs: &'a [u8]
}

impl GamePacket for CacheMissResponse<'_> {
    const ID: u32 = 0x88;
}

impl Encodable for CacheMissResponse<'_> {
    fn encode(&self) -> VResult<BytesMut> {
        todo!()

        // let mut buffer = BytesMut::with_capacity(
        //     1 + self.blobs.fold(0, |acc, blob| acc + blob.len())
        // );

        // buffer.put_var_u32(self.blobs.len() as u32);
        // for blob in self.blobs {
        //     todo!();
        //     // blob.encode(&mut buffer);
        // }

        // Ok(buffer)
    }
}