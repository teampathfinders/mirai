use util::{Result, Serialize};
use util::BinaryWrite;

use crate::bedrock::ConnectedPacket;
use crate::bedrock::CacheBlob;

#[derive(Debug, Clone)]
pub struct CacheMissResponse<'a> {
    pub blobs: &'a [CacheBlob<'a>],
}

impl ConnectedPacket for CacheMissResponse<'_> {
    const ID: u32 = 0x88;

    fn serialized_size(&self) -> usize {
        1 + self.blobs.iter().fold(0, |acc, blob| acc + blob.len())
    }
}

impl Serialize for CacheMissResponse<'_> {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_u32(self.blobs.len() as u32)?;
        for blob in self.blobs {
            blob.serialize_into(writer)?;
        }

        Ok(())
    }
}