use std::fmt::Debug;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::{fmt, io};
use crate::bytes::ToBytes;

pub struct WriteBuffer(Vec<u8>);

impl WriteBuffer {
    #[inline]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    // #[inline]
    // pub fn reserve(&mut self, additional: usize) {
    //     self.0.reserve(additional);
    // }

    #[inline]
    pub fn write_be<T: ToBytes>(&mut self, v: T)
    where
        [(); T::SIZE]:
    {
        self.0.extend_from_slice(&v.to_bytes_be());
    }

    #[inline]
    pub fn write_le<T: ToBytes>(&mut self, v: T)
    where
        [(); T::SIZE]:
    {
        self.0.extend_from_slice(&v.to_bytes_le());
    }
}

impl From<Vec<u8>> for WriteBuffer {
    #[inline]
    fn from(b: Vec<u8>) -> Self {
        Self(b)
    }
}

impl Debug for WriteBuffer {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self.0)
    }
}

impl Deref for WriteBuffer {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl DerefMut for WriteBuffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Write for WriteBuffer {
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