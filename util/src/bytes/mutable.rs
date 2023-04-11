use std::fmt::Debug;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::{fmt, io};

use super::BinaryWrite;

/// A buffer that can be read from and written to.
#[derive(Default, Clone)]
#[repr(transparent)]
pub struct BinVec(Vec<u8>);

impl BinVec {
    /// Creates a new empty buffer.
    ///
    /// This call does not allocate.
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    #[inline]
    pub fn reserve_absolute(&mut self, total: usize) {
        let capacity = self.0.capacity();
        if total > capacity {
            self.reserve(total - capacity);
        }
    }

    #[inline]
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

impl AsRef<[u8]> for BinVec {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<Vec<u8>> for BinVec {
    #[inline]
    fn from(data: Vec<u8>) -> Self {
        Self(data)
    }
}

impl Debug for BinVec {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self.0)
    }
}

impl Deref for BinVec {
    type Target = Vec<u8>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BinVec {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Write for BinVec {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.extend_from_slice(buf);
        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl BinaryWrite for BinVec {
    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
