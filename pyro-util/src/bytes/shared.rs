use crate::bail;
use crate::bytes::{BinaryReader, VarInt};
use paste::paste;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::io::Read;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Index};
use std::rc::Rc;
use std::sync::Arc;
use std::{cmp, fmt, io};

use crate::Result;

macro_rules! define_read_fns {
    ($($ty: ident),+) => {
        paste!{$(
            #[inline]
            fn [<read_ $ty _be>](&mut self) -> $crate::Result<$ty> {
                let bytes = self.take_const()?;
                Ok(<$ty>::from_be_bytes(bytes))
            }

            #[inline]
            fn [<read_ $ty _le>](&mut self) -> $crate::Result<$ty> {
                let bytes = self.take_const()?;
                Ok(<$ty>::from_le_bytes(bytes))
            }
        )+}
    }
}

pub struct ArcBuffer(Arc<Vec<u8>>);

impl Deref for ArcBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

/// Buffer that can be used to read binary data.
///
/// See [`MutableBuffer`](crate::MutableBuffer) for an owned and writable buffer.
#[derive(Copy, Clone)]
pub struct SharedBuffer<'a>(&'a [u8]);

impl<'a> SharedBuffer<'a> {
    /// Advances the cursor, skipping `n` bytes.
    #[inline]
    pub fn advance(&mut self, n: usize) {
        let (_, b) = self.0.split_at(n);
        *self = SharedBuffer::from(b);
    }
}

impl<'a> BinaryReader for &'a [u8] {
    define_read_fns!(u16, i16, u32, i32, u64, i64, u128, i128, f32, f64);

    /// Takes a specified amount of bytes from the buffer.
    ///
    /// If the amount of bytes to take from the buffer is known at compile-time,
    /// [`take_const`](Self::take_const) can be used instead.
    ///
    /// # Errors
    /// Returns [`UnexpectedEof`](Error::UnexpectedEof) if the read exceeds the buffer length.
    #[inline]
    fn take_n(&mut self, n: usize) -> Result<&[u8]> {
        if self.len() < n {
            bail!(
                UnexpectedEof,
                "expected {n} remaining bytes, got {}",
                self.len()
            )
        } else {
            let (a, b) = self.split_at(n);
            // *self = SharedBuffer::from(b);
            *self = b;
            Ok(a)
        }
    }

    /// Takes a specified amount of bytes from the buffer.
    ///
    /// This method is generic over the amount of bytes to take.
    /// In case the amount is known at compile time, this function can be used to
    /// take a sized array from the buffer.
    ///
    /// See [`take_n`](Self::take_n) for a runtime-sized alternative.
    ///
    /// # Errors
    /// Returns [`UnexpectedEof`](Error::UnexpectedEof) if the read exceeds the buffer length.
    #[inline]
    fn take_const<const N: usize>(&mut self) -> Result<[u8; N]> {
        if self.len() < N {
            bail!(
                UnexpectedEof,
                "expected {N} remaining bytes, got {}",
                self.len()
            )
        } else {
            let (a, b) = self.split_at(N);
            // *self = SharedBuffer::from(b);
            *self = b;
            // SAFETY: We can unwrap because the array is guaranteed to be the required size.
            unsafe { Ok(a.try_into().unwrap_unchecked()) }
        }
    }

    /// Takes a specified amount of bytes from the buffer without advancing the cursor.
    ///
    /// If the amount of bytes to take from the buffer is known at compile-time,
    /// [`peek_const`](Self::peek_const) can be used instead.
    ///
    /// # Errors
    /// Returns [`UnexpectedEof`](Error::UnexpectedEof) if the read exceeds the buffer length.
    #[inline]
    fn peek(&self, n: usize) -> Result<&[u8]> {
        if self.len() < n {
            bail!(
                UnexpectedEof,
                "expected {n} remaining bytes, got {}",
                self.len()
            )
        } else {
            Ok(&self[..n])
        }
    }

    /// Takes a specified amount of bytes from the buffer.
    ///
    /// This method is generic over the amount of bytes to take.
    /// In case the amount is known at compile time, this function can be used to
    /// take a sized array from the buffer.
    ///
    /// See [`peek`](Self::peek) for a runtime-sized alternative.
    ///
    /// # Errors
    /// Returns [`UnexpectedEof`](Error::UnexpectedEof) if the read exceeds the buffer length.
    #[inline]
    fn peek_const<const N: usize>(&self) -> Result<[u8; N]> {
        if self.len() < N {
            bail!(
                UnexpectedEof,
                "expected {N} remaining bytes, got {}",
                self.len()
            )
        } else {
            let dst = &self[..N];
            // SAFETY: dst is guaranteed to be of length N
            // due to the slicing above which already implements bounds checks.
            unsafe { Ok(dst.try_into().unwrap_unchecked()) }
        }
    }

    #[inline]
    fn read_bool(&mut self) -> Result<bool> {
        Ok(self.take_const::<1>()?[0] != 0)
    }

    #[inline]
    fn read_u8(&mut self) -> Result<u8> {
        Ok(self.take_const::<1>()?[0])
    }

    #[inline]
    fn read_i8(&mut self) -> Result<i8> {
        Ok(self.take_const::<1>()?[0] as i8)
    }

    fn read_var_u32(&mut self) -> Result<u32> {
        let mut v = 0;
        let mut i = 0;
        while i < 35 {
            let b = self.read_u8()?;
            v |= ((b & 0x7f) as u32) << i;
            if b & 0x80 == 0 {
                return Ok(v);
            }
            i += 7;
        }

        bail!(
            Malformed,
            "variable 32-bit integer did not end after 5 bytes"
        )
    }

    fn read_var_u64(&mut self) -> Result<u64> {
        let mut v = 0;
        let mut i = 0;
        while i < 70 {
            let b = self.read_u8()?;
            v |= ((b & 0x7f) as u64) << i;

            if b & 0x80 == 0 {
                return Ok(v);
            }

            i += 7;
        }

        bail!(
            Malformed,
            "variable 64-bit integer did not end after 10 bytes"
        )
    }

    fn read_var_i32(&mut self) -> Result<i32> {
        let vx = self.read_var_u32()?;
        let mut v = (vx >> 1) as i32;

        if vx & 1 != 0 {
            v = !v;
        }

        Ok(v)
    }

    fn read_var_i64(&mut self) -> Result<i64> {
        let vx = self.read_var_u64()?;
        let mut v = (vx >> 1) as i64;

        if vx & 1 != 0 {
            v = !v;
        }

        Ok(v)
    }

    fn read_str(&mut self) -> Result<&str> {
        let len = self.read_var_u32()?;
        let data = self.take_n(len as usize)?;

        Ok(std::str::from_utf8(data)?)
    }
}

impl<'a> From<&'a [u8]> for SharedBuffer<'a> {
    #[inline]
    fn from(b: &'a [u8]) -> Self {
        Self(b)
    }
}

impl<'a> Deref for SharedBuffer<'a> {
    type Target = &'a [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for SharedBuffer<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> Index<usize> for SharedBuffer<'a> {
    type Output = u8;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

// impl<'a> IntoIterator for ReadBuffer<'a> {
//     type Item = &'a u8;
//     type IntoIter = std::slice::Iter<'a, u8>;

//     #[inline]
//     fn into_iter(self) -> Self::IntoIter {
//         self.0.iter()
//     }
// }

impl<'a> Debug for SharedBuffer<'a> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self.0)
    }
}

impl<'a> Read for SharedBuffer<'a> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let amt = cmp::min(self.len(), buf.len());
        let (a, b) = self.0.split_at(amt);

        if amt == 1 {
            buf[0] = a[0];
        } else {
            buf[..amt].copy_from_slice(a);
        }

        *self = SharedBuffer::from(b);
        Ok(amt)
    }
}

#[cfg(test)]
mod test {
    use super::SharedBuffer;
    use crate::bytes::BinaryReader;

    #[test]
    fn test_read_u8() {
        let s: &[u8] = &[42, 12, 1, 2, 3];
        let mut buf = SharedBuffer::from(s);

        for x in s {
            assert_eq!(buf.read_u8().unwrap(), *x);
        }
    }

    #[test]
    fn test_read_i8() {
        let s: &[i8] = &[-10, 5, -42, 120];
        let mut buf = SharedBuffer::from(unsafe {
            std::mem::transmute::<&[i8], &[u8]>(s)
        });

        for x in s {
            assert_eq!(buf.read_i8().unwrap(), *x);
        }
    }

    #[test]
    fn test_read_u16() {
        let o = [42, 24083];
        let s: &[u8] = &[0, 42, 94, 19];
        let mut buf = SharedBuffer::from(s);

        for x in o {
            assert_eq!(buf.read_u16_be().unwrap(), x);
        }
    }

    #[test]
    fn test_read_i16() {
        let o = [-2397, 24083];
        let s: &[u8] = &[246, 163, 94, 19];
        let mut buf = SharedBuffer::from(s);

        for x in o {
            assert_eq!(buf.read_i16_be().unwrap(), x);
        }
    }

    #[test]
    fn test_read_str() {
        let o = "Hello, World!";
        let s: &[u8] = &[
            13, 72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33,
        ];
        let mut buf = SharedBuffer::from(s);

        assert_eq!(buf.read_str().unwrap(), o);
        assert!(buf.is_empty());
    }
}
