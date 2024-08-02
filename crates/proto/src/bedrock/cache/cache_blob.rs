use std::sync::Arc;

use util::{BinaryWrite, RVec, Serialize};

/// A blob used in the cache protocol.
#[derive(Debug, Clone)]
pub struct CacheBlob {
    /// Hash of the payload, computed with xxHash.
    pub hash: u64,
    /// Payload of the blob.
    pub payload: Arc<RVec>, // pub payload: &'a [u8],
}

impl Serialize for CacheBlob {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_u64_le(self.hash)?;
        writer.write_var_u32(self.payload.len() as u32)?;
        writer.write_all(&self.payload)?;

        Ok(())
    }
}

impl CacheBlob {
    #[inline]
    pub fn len(&self) -> usize {
        8 + self.payload.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
