use std::sync::Arc;

use util::BinaryWrite;
use util::Serialize;

use crate::bedrock::CacheBlob;
use crate::bedrock::ConnectedPacket;

#[derive(Debug, Clone)]
pub struct CacheMissResponse<'a> {
    pub blobs: &'a [CacheBlob],
}

impl<'a> ConnectedPacket for CacheMissResponse<'a> {
    const ID: u32 = 0x88;

    fn serialized_size(&self) -> usize {
        1 + self.blobs.iter().fold(0, |acc, blob| acc + blob.len())
    }
}

impl<'a> Serialize for CacheMissResponse<'a> {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_u32(self.blobs.len() as u32)?;
        for blob in self.blobs {
            blob.serialize_into(writer)?;
        }

        Ok(())
    }
}
