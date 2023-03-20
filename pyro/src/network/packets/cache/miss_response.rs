
use util::{Serialize, Result};
use util::bytes::{BinaryWrite, MutableBuffer};
use crate::ConnectedPacket;
use crate::network::cache_blob::CacheBlob;

#[derive(Debug, Clone)]
pub struct CacheMissResponse<'a> {
    pub blobs: &'a [CacheBlob<'a>]
}

impl ConnectedPacket for CacheMissResponse<'_> {
    const ID: u32 = 0x88;

    fn serialized_size(&self) -> usize {
        1 + self.blobs.iter().fold(0, |acc, blob| acc + blob.len())
    }
}

impl Serialize for CacheMissResponse<'_> {
    fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_var_u32(self.blobs.len() as u32)?;
        for blob in self.blobs {
            blob.serialize(buffer)?;
        }

        Ok(())
    }
}