use std::fmt::Debug;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::{fmt, io};
use crate::{bail, Result};
use crate::bytes::{BinaryBuffer, FromBytes, ToBytes, VarInt};

/// A buffer that can be read from and written to.
/// It is the owned version of [`ReadBuffer`].
pub struct OwnedBuffer(Vec<u8>);

impl OwnedBuffer {
    #[inline]
    pub fn new() -> Self {
        Self(Vec::new())
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
    pub fn copy_from_slice(slice: &[u8]) -> Self {
        Self(slice.to_vec())
    }

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

// impl BinaryBuffer for OwnedBuffer {
    // /// Reads the specified big-endian encoded type from the buffer without advancing the cursor.
    // #[inline]
    // fn peek_be<T: FromBytes>(&self) -> Result<T>
    //     where
    //         [(); T::SIZE]:,
    // {
    //     Ok(T::from_be(self.peek_const::<{ T::SIZE }>()?))
    // }
    //
    // /// Reads the specified little-endian encoded type from the buffer without advancing the cursor.
    // #[inline]
    // fn peek_le<T: FromBytes>(&self) -> Result<T>
    //     where
    //         [(); T::SIZE]:,
    // {
    //     Ok(T::from_le(self.peek_const::<{ T::SIZE }>()?))
    // }

    /// Takes a specified amount of bytes from the buffer.
    ///
    /// If the amount of bytes to take from the buffer is known at compile-time,
    /// [`take_const`](Self::take_const) can be used instead.
    ///
    /// # Errors
    /// Returns [`UnexpectedEof`](Error::UnexpectedEof) if the read exceeds the buffer length.
    // #[inline]
    // fn take(&mut self, n: usize) -> Result<&[u8]> {
    //     if self.len() < n {
    //         bail!(
    //             UnexpectedEof,
    //             "expected {n} remaining bytes, got {}",
    //             self.len()
    //         )
    //     } else {
    //         let (a, b) = self.0.split_at(n);
    //         *self = OwnedBuffer::from(b.to_vec());
    //         Ok(a)
    //     }
    // }
    //
    // /// Takes a specified amount of bytes from the buffer.
    // ///
    // /// This method is generic over the amount of bytes to take.
    // /// In case the amount is known at compile time, this function can be used to
    // /// take a sized array from the buffer.
    // ///
    // /// Unlike the implementation on [`SharedBuffer`], this implementation will allocate
    // ///
    // /// See [`take`](Self::take) for a runtime-sized alternative.
    // ///
    // /// # Errors
    // /// Returns [`UnexpectedEof`](Error::UnexpectedEof) if the read exceeds the buffer length.
    // #[inline]
    // fn take_const<const N: usize>(&mut self) -> Result<[u8; N]> {
    //     if self.len() < N {
    //         bail!(
    //             UnexpectedEof,
    //             "expected {N} remaining bytes, got {}",
    //             self.len()
    //         )
    //     } else {
    //         let (a, b) = self.0.split_at(N);
    //         *self = OwnedBuffer::from(b.to_vec());
    //         // SAFETY: We can unwrap because the array is guaranteed to be the required size.
    //         unsafe { Ok(a.try_into().unwrap_unchecked()) }
    //     }
    // }
    //
    // /// Takes a specified amount of bytes from the buffer without advancing the cursor.
    // ///
    // /// If the amount of bytes to take from the buffer is known at compile-time,
    // /// [`peek_const`](Self::peek_const) can be used instead.
    // ///
    // /// # Errors
    // /// Returns [`UnexpectedEof`](Error::UnexpectedEof) if the read exceeds the buffer length.
    // #[inline]
    // fn peek(&self, n: usize) -> Result<&[u8]> {
    //     if self.len() < n {
    //         bail!(
    //             UnexpectedEof,
    //             "expected {n} remaining bytes, got {}",
    //             self.len()
    //         )
    //     } else {
    //         Ok(&self.0[..n])
    //     }
    // }
    //
    // /// Takes a specified amount of bytes from the buffer.
    // ///
    // /// This method is generic over the amount of bytes to take.
    // /// In case the amount is known at compile time, this function can be used to
    // /// take a sized array from the buffer.
    // ///
    // /// See [`peek`](Self::peek) for a runtime-sized alternative.
    // ///
    // /// # Errors
    // /// Returns [`UnexpectedEof`](Error::UnexpectedEof) if the read exceeds the buffer length.
    // #[inline]
    // fn peek_const<const N: usize>(&self) -> Result<[u8; N]> {
    //     if self.len() < N {
    //         bail!(
    //             UnexpectedEof,
    //             "expected {N} remaining bytes, got {}",
    //             self.len()
    //         )
    //     } else {
    //         let dst = &self.0[..N];
    //         // SAFETY: dst is guaranteed to be of length N
    //         // due to the slicing above which already implements bounds checks.
    //         unsafe { Ok(dst.try_into().unwrap_unchecked()) }
    //     }
    // }
    //
    // /// Reads a little-endian encoded type from the buffer.
    // ///
    // /// See [`FromBytes`] for a list of types that can be read from the buffer with this method.
    // #[inline]
    // fn read_le<T: FromBytes>(&mut self) -> Result<T>
    //     where
    //         [(); T::SIZE]:,
    // {
    //     let bytes = self.take_const::<{ T::SIZE }>()?;
    //     Ok(T::from_le(bytes))
    // }
    //
    // /// Reads a big-endian encoded type from the buffer.
    // ///
    // /// See [`FromBytes`] for a list of types that can be read from the buffer with this method.
    // #[inline]
    // fn read_be<T: FromBytes>(&mut self) -> Result<T>
    //     where
    //         [(); T::SIZE]:,
    // {
    //     let bytes = self.take_const::<{ T::SIZE }>()?;
    //     Ok(T::from_be(bytes))
    // }
    //
    // /// Reads a variable size integer from the buffer.
    // /// See [`VarInt`] for a list of available types.
    // #[inline]
    // fn read_var<T>(&mut self) -> Result<T>
    // where
    //     T: VarInt,
    // {
    //     T::read(self)
    // }
// }

impl From<Vec<u8>> for OwnedBuffer {
    #[inline]
    fn from(b: Vec<u8>) -> Self {
        Self(b)
    }
}

impl Debug for OwnedBuffer {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self.0)
    }
}

impl Deref for OwnedBuffer {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl DerefMut for OwnedBuffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Write for OwnedBuffer {
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