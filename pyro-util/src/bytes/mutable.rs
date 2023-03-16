use crate::bytes::{BinaryReader, BinaryWriter, SharedBuffer, VarInt};
use crate::{bail, BlockPosition, Result, Vector};
use paste::paste;
use std::borrow::Cow;
use std::fmt::Debug;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::{fmt, io};
use uuid::Uuid;

/// A buffer that can be read from and written to.
/// It is the owned version of [`ReadBuffer`].
#[derive(Default)]
pub struct MutableBuffer {
    data: Vec<u8>,
    cursor: usize,
}

impl MutableBuffer {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn insert(&mut self, index: usize, element: u8) {
        self.data.insert(index, element);
    }

    #[inline]
    pub fn advance_cursor(&mut self, n: usize) {
        self.cursor += n;
    }

    #[inline]
    pub fn truncate(&mut self, n: usize) {
        self.data.truncate(n);
    }

    #[inline]
    pub fn clear(&mut self) {
        self.data.clear();
        self.cursor = 0;
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            cursor: 0,
        }
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }

    #[inline]
    pub fn reserve_to(&mut self, total: usize) {
        if total > self.data.capacity() {
            self.reserve(total - self.data.capacity());
        }
    }

    #[inline]
    pub fn inner(&self) -> &Vec<u8> {
        &self.data
    }

    #[inline]
    pub fn snapshot(&self) -> SharedBuffer {
        // SharedBuffer::from(&self.as_slice()[self.cursor..])
        SharedBuffer::from(self.as_slice())
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data.as_mut_slice()[self.cursor..]
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.data.as_slice()[self.cursor..]
    }

    #[inline]
    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }
}

impl AsRef<[u8]> for MutableBuffer {
    fn as_ref(&self) -> &[u8] {
        self.data.as_ref()
    }
}

impl BinaryWriter for MutableBuffer {
    #[inline]
    fn append(&mut self, v: &[u8]) {
        self.data.extend_from_slice(v);
    }
}

impl From<Vec<u8>> for MutableBuffer {
    #[inline]
    fn from(data: Vec<u8>) -> Self {
        Self { data, cursor: 0 }
    }
}

impl Debug for MutableBuffer {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self.data)
    }
}

impl Deref for MutableBuffer {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.data.as_slice()
    }
}

impl DerefMut for MutableBuffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data.as_mut_slice()
    }
}

impl Write for MutableBuffer {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.data.extend_from_slice(buf);
        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
