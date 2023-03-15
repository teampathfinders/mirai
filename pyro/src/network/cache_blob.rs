/// A blob used in the cache protocol.
#[derive(Debug, Clone)]
pub struct CacheBlob {
    /// Hash of the payload, computed with xxHash.
    pub hash: u64,
    /// Payload of the blob.
    pub payload: SharedBuffer,
}

impl CacheBlob {
    pub fn serialize(&self, buffer: &mut OwnedBuffer) {
        buffer.put_u64_le(self.hash);
        buffer.extend(&self.payload);
    }

    #[inline]
    pub const fn len(&self) -> usize {
        std::mem::size_of::<u64>() + self.payload.len()
    }
}
