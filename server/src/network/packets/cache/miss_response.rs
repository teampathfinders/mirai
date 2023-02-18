use bytes::BytesMut;
use common::{Serialize, VResult, WriteExtensions};
use crate::network::cache_blob::CacheBlob;
use crate::network::packets::GamePacket;

#[derive(Debug, Clone)]
pub struct CacheMissResponse<'a> {
    pub blobs: &'a [CacheBlob]
}

impl GamePacket for CacheMissResponse<'_> {
    const ID: u32 = 0x88;
}

impl Serialize for CacheMissResponse<'_> {
    fn serialize(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(
            1 + self.blobs.iter().fold(0, |acc, blob| acc + blob.len())
        );

        buffer.put_var_u32(self.blobs.len() as u32);
        for blob in self.blobs {
            blob.encode(&mut buffer);
        }

        Ok(buffer)
    }
}