use util::BinaryWrite;

/// A blob used in the cache protocol.
#[derive(Debug, Clone)]
pub struct CacheBlob<'a> {
    /// Hash of the payload, computed with xxHash.
    pub hash: u64,
    /// Payload of the blob.
    pub payload: &'a [u8],
}

impl<'a> CacheBlob<'a> {
    pub fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_u64_le(self.hash)?;
        writer.write_all(&self.payload)?;

        Ok(())
    }

    #[inline]
    pub const fn len(&self) -> usize {
        8 + self.payload.len()
    }
}
