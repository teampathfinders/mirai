use crate::{bail, Error};
use std::fmt::Debug;
use std::io::Read;
use std::ops::{Deref, Index};
use std::{cmp, fmt, io};
use crate::bytes::FromBytes;

use crate::Result;

/// Buffer that can be used to read binary data.
///
/// See [`WriteBuffer`](crate::WriteBuffer) for a writable buffer.
pub struct ReadBuffer<'a>(&'a [u8]);

impl<'a> ReadBuffer<'a> {
    /// Advances the cursor, skipping `n` bytes.
    #[inline]
    pub fn advance(&mut self, n: usize) {
        let (_, b) = self.0.split_at(n);
        *self = ReadBuffer::from(b);
    }

    /// Reads the specified big-endian encoded type from the buffer without advancing the cursor.
    #[inline]
    pub fn peek_be<T: FromBytes>(&self) -> Result<T>
    where
        [(); T::SIZE]:,
    {
        Ok(T::from_be_bytes(self.peek_const::<{ T::SIZE }>()?))
    }

    /// Reads the specified little-endian encoded type from the buffer without advancing the cursor.
    #[inline]
    pub fn peek_le<T: FromBytes>(&self) -> Result<T>
    where
        [(); T::SIZE]:,
    {
        Ok(T::from_le_bytes(self.peek_const::<{ T::SIZE }>()?))
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
    pub fn peek_const<const N: usize>(&self) -> Result<[u8; N]> {
        if self.len() < N {
            bail!(
                UnexpectedEof,
                "expected {N} remaining bytes, got {}",
                self.len()
            )
        } else {
            let dst = &self.0[..N];
            // SAFETY: dst is guaranteed to be of length N
            // due to the slicing above which already implements bounds checks.
            unsafe { Ok(dst.try_into().unwrap_unchecked()) }
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
    pub fn peek(&self, n: usize) -> Result<&[u8]> {
        if self.len() < n {
            bail!(
                UnexpectedEof,
                "expected {n} remaining bytes, got {}",
                self.len()
            )
        } else {
            Ok(&self.0[..n])
        }
    }

    /// Takes a specified amount of bytes from the buffer.
    ///
    /// If the amount of bytes to take from the buffer is known at compile-time,
    /// [`take_const`](Self::take_const) can be used instead.
    ///
    /// # Errors
    /// Returns [`UnexpectedEof`](Error::UnexpectedEof) if the read exceeds the buffer length.
    #[inline]
    pub fn take(&mut self, n: usize) -> Result<&[u8]> {
        if self.len() < n {
            bail!(
                UnexpectedEof,
                "expected {n} remaining bytes, got {}",
                self.len()
            )
        } else {
            let (a, b) = self.0.split_at(n);
            *self = ReadBuffer::from(b);
            Ok(a)
        }
    }

    /// Takes a specified amount of bytes from the buffer.
    ///
    /// This method is generic over the amount of bytes to take.
    /// In case the amount is known at compile time, this function can be used to
    /// take a sized array from the buffer.
    ///
    /// See [`take`](Self::take) for a runtime-sized alternative.
    ///
    /// # Errors
    /// Returns [`UnexpectedEof`](Error::UnexpectedEof) if the read exceeds the buffer length.
    #[inline]
    pub fn take_const<const N: usize>(&mut self) -> Result<[u8; N]> {
        if self.len() < N {
            bail!(
                UnexpectedEof,
                "expected {N} remaining bytes, got {}",
                self.len()
            )
        } else {
            let (a, b) = self.0.split_at(N);
            *self = ReadBuffer::from(b);
            // SAFETY: We can unwrap because the array is guaranteed to be the required size.
            unsafe { Ok(a.try_into().unwrap_unchecked()) }
        }
    }

    /// Reads a big-endian encoded type from the buffer.
    ///
    /// See [`FromBytes`] for a list of types that can be read from the buffer with this method.
    #[inline]
    pub fn read_be<T: FromBytes>(&mut self) -> Result<T>
    where
        [(); T::SIZE]:,
    {
        let bytes = self.take_const::<{ T::SIZE }>()?;
        Ok(T::from_be_bytes(bytes))
    }

    /// Reads a little-endian encoded type from the buffer.
    ///
    /// See [`FromBytes`] for a list of types that can be read from the buffer with this method.
    #[inline]
    pub fn read_le<T: FromBytes>(&mut self) -> Result<T>
    where
        [(); T::SIZE]:,
    {
        let bytes = self.take_const::<{ T::SIZE }>()?;
        Ok(T::from_le_bytes(bytes))
    }
}

impl<'a> From<&'a [u8]> for ReadBuffer<'a> {
    #[inline]
    fn from(b: &'a [u8]) -> Self {
        Self(b)
    }
}

impl<'a> Deref for ReadBuffer<'a> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> Index<usize> for ReadBuffer<'a> {
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

impl<'a> Debug for ReadBuffer<'a> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self.0)
    }
}

impl<'a> Read for ReadBuffer<'a> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let amt = cmp::min(self.len(), buf.len());
        let (a, b) = self.0.split_at(amt);

        if amt == 1 {
            buf[0] = a[0];
        } else {
            buf[..amt].copy_from_slice(a);
        }

        *self = ReadBuffer::from(b);
        Ok(amt)
    }
}

#[cfg(test)]
mod test {
    use super::ReadBuffer;

    #[test]
    fn test_read_u8() {
        let s: &[u8] = &[42, 12, 1, 2, 3];
        let mut buf = ReadBuffer::from(s);

        for x in s {
            assert_eq!(buf.peek_be::<u8>().unwrap(), *x);
            assert_eq!(buf.read_be::<u8>().unwrap(), *x);
        }
    }

    #[test]
    fn test_read_i8() {
        let s: &[i8] = &[-10, 5, -42, 120];
        let mut buf =
            ReadBuffer::from(unsafe { std::mem::transmute::<&[i8], &[u8]>(s) });

        for x in s {
            assert_eq!(buf.peek_be::<i8>().unwrap(), *x);
            assert_eq!(buf.read_be::<i8>().unwrap(), *x);
        }
    }

    #[test]
    fn test_read_u16() {
        let o = [42, 24083];
        let s: &[u8] = &[0, 42, 94, 19];
        let mut buf = ReadBuffer::from(s);

        for x in o {
            assert_eq!(buf.peek_be::<u16>().unwrap(), x);
            assert_eq!(buf.read_be::<u16>().unwrap(), x);
        }
    }

    #[test]
    fn test_read_i16() {
        let o = [-2397, 24083];
        let s: &[u8] = &[246, 163, 94, 19];
        let mut buf = ReadBuffer::from(s);

        for x in o {
            assert_eq!(buf.peek_be::<i16>().unwrap(), x);
            assert_eq!(buf.read_be::<i16>().unwrap(), x);
        }
    }
}
