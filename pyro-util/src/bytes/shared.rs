use crate::bytes::{BinaryReader, MutableBuffer, VarInt};
use crate::{bail, BlockPosition};
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

#[derive(Debug, Clone)]
pub struct ArcBuffer(Arc<Vec<u8>>);

impl ArcBuffer {
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl From<MutableBuffer> for ArcBuffer {
    #[inline]
    fn from(value: MutableBuffer) -> Self {
        ArcBuffer(Arc::new(value.into_inner()))
    }
}

impl AsRef<[u8]> for ArcBuffer {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl Deref for ArcBuffer {
    type Target = [u8];

    #[inline]
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

    #[inline]
    pub fn truncate(&mut self, n: usize) {
        let (a, _) = self.0.split_at(n);
        *self = SharedBuffer::from(a);
    }
}

impl<'a> BinaryReader<'a> for &'a [u8] {
    fn advance(&mut self, n: usize) -> Result<()> {
        if self.len() < n {
            bail!(
                UnexpectedEof,
                "cannot advance past {n} bytes, remaining: {}",
                self.len()
            )
        }

        let (_, b) = self.split_at(n);
        *self = b;

        Ok(())
    }

    /// Takes a specified amount of bytes from the buffer.
    ///
    /// If the amount of bytes to take from the buffer is known at compile-time,
    /// [`take_const`](Self::take_const) can be used instead.
    ///
    /// # Errors
    /// Returns [`UnexpectedEof`](Error::UnexpectedEof) if the read exceeds the buffer length.
    #[inline]
    fn take_n(&mut self, n: usize) -> Result<&'a [u8]> {
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
    use crate::bytes::SharedBuffer;
    use crate::bytes::{BinaryReader, BinaryWriter, MutableBuffer};
    use crate::u24::u24;
    use paste::paste;

    macro_rules! define_test_fns {
        ($($ty: ident),+) => {
            paste! {$(
                #[test]
                fn [<read_write_ $ty _ le>]() {
                    const VALUES: [$ty; 4] = [$ty::MAX, $ty::MIN, $ty::MAX - 42, $ty::MIN + 42];

                    let mut buffer = MutableBuffer::new();
                    for v in VALUES {
                        buffer.[<write_ $ty _le>](v);
                    }

                    let mut ss = buffer.snapshot();
                    for v in VALUES {
                        assert_eq!(v, ss.[<read_ $ty _le>]().unwrap());
                    }
                }

                #[test]
                fn [<read_write_ $ty _ be>]() {
                    const VALUES: [$ty; 4] = [$ty::MAX, $ty::MIN, $ty::MAX - 42, $ty::MIN + 42];

                    let mut buffer = MutableBuffer::new();
                    for v in VALUES {
                        buffer.[<write_ $ty _be>](v);
                    }

                    let mut ss = buffer.snapshot();
                    for v in VALUES {
                        assert_eq!(v, ss.[<read_ $ty _be>]().unwrap());
                    }
                }
            )+}
        };
    }

    define_test_fns!(u16, i16, u32, i32, u64, i64, u128, i128);

    #[test]
    fn read_write_u8() {
        const VALUES: [u8; 4] = [u8::MAX, u8::MIN, u8::MAX - 42, u8::MIN + 42];

        let mut buffer = MutableBuffer::new();
        for v in VALUES {
            buffer.write_u8(v);
        }

        let mut ss = buffer.snapshot();
        for v in VALUES {
            assert_eq!(v, ss.read_u8().unwrap());
        }
    }

    #[test]
    fn read_write_i8() {
        const VALUES: [i8; 4] = [i8::MAX, i8::MIN, i8::MAX - 42, i8::MIN + 42];

        let mut buffer = MutableBuffer::new();
        for v in VALUES {
            buffer.write_i8(v);
        }

        let mut ss = buffer.snapshot();
        for v in VALUES {
            assert_eq!(v, ss.read_i8().unwrap());
        }
    }

    // #[test]
    // fn read_write_u24_le() {
    //     const VALUES: [u24; 4] = [u24::MAX, u24::MIN, u24::MAX - 42.0, u24::MIN + 42.0];
    //
    //     let mut buffer = MutableBuffer::new();
    //     for v in VALUES {
    //         buffer.write_u24_le(v);
    //     }
    //
    //     let mut ss = buffer.snapshot();
    //     for v in VALUES {
    //         assert_eq!(v, ss.read_u24_le().unwrap());
    //     }
    // }
    //
    // #[test]
    // fn read_write_u24_be() {
    //     const VALUES: [u24; 4] = [u24::MAX, u24::MIN, u24::MAX - 42.0, u24::MIN + 42.0];
    //
    //     let mut buffer = MutableBuffer::new();
    //     for v in VALUES {
    //         buffer.write_u24_be(v);
    //     }
    //
    //     let mut ss = buffer.snapshot();
    //     for v in VALUES {
    //         assert_eq!(v, ss.read_u24_be().unwrap());
    //     }
    // }

    #[test]
    fn read_write_f32_le() {
        const VALUES: [f32; 4] =
            [f32::MAX, f32::MIN, f32::MAX - 42.0, f32::MIN + 42.0];

        let mut buffer = MutableBuffer::new();
        for v in VALUES {
            buffer.write_f32_le(v);
        }

        let mut ss = buffer.snapshot();
        for v in VALUES {
            assert_eq!(v, ss.read_f32_le().unwrap());
        }
    }

    #[test]
    fn read_write_f32_be() {
        const VALUES: [f32; 4] =
            [f32::MAX, f32::MIN, f32::MAX - 42.0, f32::MIN + 42.0];

        let mut buffer = MutableBuffer::new();
        for v in VALUES {
            buffer.write_f32_be(v);
        }

        let mut ss = buffer.snapshot();
        for v in VALUES {
            assert_eq!(v, ss.read_f32_be().unwrap());
        }
    }

    #[test]
    fn read_write_f64_le() {
        const VALUES: [f64; 4] =
            [f64::MAX, f64::MIN, f64::MAX - 42.0, f64::MIN + 42.0];

        let mut buffer = MutableBuffer::new();
        for v in VALUES {
            buffer.write_f64_le(v);
        }

        let mut ss = buffer.snapshot();
        for v in VALUES {
            assert_eq!(v, ss.read_f64_le().unwrap());
        }
    }

    #[test]
    fn read_write_f64_be() {
        const VALUES: [f64; 4] =
            [f64::MAX, f64::MIN, f64::MAX - 42.0, f64::MIN + 42.0];

        let mut buffer = MutableBuffer::new();
        for v in VALUES {
            buffer.write_f64_be(v);
        }

        let mut ss = buffer.snapshot();
        for v in VALUES {
            assert_eq!(v, ss.read_f64_be().unwrap());
        }
    }

    #[test]
    fn read_write_str() {
        let mut buffer = MutableBuffer::new();
        buffer.write_str("Hello, World!");

        let mut ss = buffer.snapshot();
        assert_eq!(ss.read_str().unwrap(), "Hello, World!");
    }
}
