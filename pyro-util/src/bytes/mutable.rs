use crate::bytes::{BinaryReader, BinaryWriter, SharedBuffer, VarInt};
use crate::{bail, Result};
use paste::paste;
use std::borrow::Cow;
use std::fmt::Debug;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::{fmt, io};

macro_rules! define_write_fns {
    ($($ty: ident),+) => {
        paste! {$(
            #[inline]
            fn [<write_ $ty _le>](&mut self, v: $ty) {
                let bytes = v.to_le_bytes();
                self.0.extend_from_slice(&bytes);
            }

            #[inline]
            fn [<write_ $ty _be>](&mut self, v: $ty) {
                let bytes = v.to_be_bytes();
                self.0.extend_from_slice(&bytes);
            }
        )+}
    }
}

/// A buffer that can be read from and written to.
/// It is the owned version of [`ReadBuffer`].
pub struct MutableBuffer(Vec<u8>);

impl MutableBuffer {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    #[inline]
    pub fn append(&mut self, v: &[u8]) {
        self.0.extend_from_slice(v);
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
    pub fn reserve_to(&mut self, total: usize) {
        if total > self.0.capacity() {
            self.reserve(self.0.capacity() - total);
        }
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

impl BinaryWriter for MutableBuffer {
    define_write_fns!(u16, i16, u32, i32, u64, i64, u128, i128, f32, f64);

    #[inline]
    fn write_bool(&mut self, v: bool) {
        self.append(&[v as u8]);
    }

    #[inline]
    fn write_u8(&mut self, v: u8) {
        self.append(&[v])
    }

    #[inline]
    fn write_i8(&mut self, v: i8) {
        self.append(&[v as u8]);
    }

    #[inline]
    fn write_var_u32(&mut self, mut v: u32) {
        while v >= 0x80 {
            self.write_u8((v as u8) | 0x80);
            v >>= 7;
        }
        self.write_u8(v as u8);
    }

    #[inline]
    fn write_var_u64(&mut self, mut v: u64) {
        while v >= 0x80 {
            self.write_u8((v as u8) | 0x80);
            v >>= 7;
        }
        self.write_u8(v as u8);
    }

    #[inline]
    fn write_var_i32(&mut self, v: i32) {
        let mut ux = (v as u32) << 1;
        if v < 0 {
            ux = !ux;
        }

        self.write_var_u32(ux);
    }

    #[inline]
    fn write_var_i64(&mut self, v: i64) {
        let mut ux = (v as u64) << 1;
        if v < 0 {
            ux = !ux;
        }

        self.write_var_u64(ux);
    }

    #[inline]
    fn write_str(&mut self, v: &str) {
        self.write_var_u32(v.len() as u32);
        self.append(v.as_bytes());
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
