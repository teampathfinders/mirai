use std::io::Write;
use util::bytes::{BinaryWriter, MutableBuffer, SharedBuffer};

/// A blob used in the cache protocol.
#[derive(Debug, Clone)]
pub struct CacheBlob<'a> {
    /// Hash of the payload, computed with xxHash.
    pub hash: u64,
    /// Payload of the blob.
    pub payload: SharedBuffer<'a>,
}

impl<'a> CacheBlob<'a> {
    pub fn serialize(&self, buffer: &mut MutableBuffer) {
        buffer.write_u64_le(self.hash);
        buffer.write(&self.payload);
    }

    #[inline]
    pub const fn len(&self) -> usize {
        8 + self.payload.len()
    }
}
