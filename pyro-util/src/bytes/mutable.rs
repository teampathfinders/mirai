use crate::bytes::{BinRead, SharedBuffer, VarInt};
use crate::{bail, Result};
use std::borrow::Cow;
use std::fmt::Debug;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::{fmt, io};

/// A buffer that can be read from and written to.
/// It is the owned version of [`ReadBuffer`].
pub struct MutableBuffer(Vec<u8>);

impl MutableBuffer {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    #[inline]
    pub fn snapshot(&self) -> SharedBuffer {
        SharedBuffer::from(self.as_slice())
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<Vec<u8>> for MutableBuffer {
    #[inline]
    fn from(b: Vec<u8>) -> Self {
        Self(b)
    }
}

impl Default for MutableBuffer {
    #[inline]
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl Debug for MutableBuffer {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self.0)
    }
}

impl Deref for MutableBuffer {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Write for MutableBuffer {
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
