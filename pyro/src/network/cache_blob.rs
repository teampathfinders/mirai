use std::io::Write;
use util::bytes::{BinaryWrite, MutableBuffer, SharedBuffer};
use util::Result;

/// A blob used in the cache protocol.
#[derive(Debug, Clone)]
pub struct CacheBlob<'a> {
    /// Hash of the payload, computed with xxHash.
    pub hash: u64,
    /// Payload of the blob.
    pub payload: SharedBuffer<'a>,
}

impl<'a> CacheBlob<'a> {
    pub fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_u64_le(self.hash)?;
        buffer.write_all(&self.payload)?;

        Ok(())
    }

    #[inline]
    pub fn len(&self) -> usize {
        8 + self.payload.len()
    }
}
