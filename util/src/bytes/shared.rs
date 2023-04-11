use std::fmt::Debug;
use std::io::Read;
use std::ops::{Deref, DerefMut, Index};
use std::sync::Arc;
use std::{cmp, fmt, io};

use crate::bytes::BinaryRead;

#[derive(Debug, Clone)]
pub struct ArcBuffer(Arc<Vec<u8>>);

impl ArcBuffer {
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl From<Vec<u8>> for ArcBuffer {
    #[inline]
    fn from(value: Vec<u8>) -> Self {
        ArcBuffer(Arc::new(value))
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

impl<'a> BinaryRead<'a> for &'a [u8] {
    fn as_slice(&self) -> &[u8] {
        self
    }

    fn advance(&mut self, n: usize) -> anyhow::Result<()> {
        if self.len() < n {
            anyhow::bail!(format!(
                "Cannot advance past {n} bytes, remaining: {}",
                self.len()
            ))
        }

        let (_, b) = self.split_at(n);
        *self = b;

        Ok(())
    }

    #[inline]
    fn remaining(&self) -> usize {
        self.len()
    }

    /// Takes a specified amount of bytes from the buffer.
    ///
    /// If the amount of bytes to take from the buffer is known at compile-time,
    /// [`take_const`](Self::take_const) can be used instead.
    ///
    /// # Errors
    /// Returns [`UnexpectedEof`](Error::UnexpectedEof) if the read exceeds the buffer length.
    #[inline]
    fn take_n(&mut self, n: usize) -> anyhow::Result<&'a [u8]> {
        if self.len() < n {
            anyhow::bail!(format!(
                "Expected {n} remaining bytes, got {}",
                self.len()
            ))
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
    fn take_const<const N: usize>(&mut self) -> anyhow::Result<[u8; N]> {
        if self.len() < N {
            anyhow::bail!(format!(
                "Expected {N} remaining bytes, got {}",
                self.len()
            ))
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
    fn peek(&self, n: usize) -> anyhow::Result<&[u8]> {
        if self.len() < n {
            anyhow::bail!(format!(
                "Expected {n} remaining bytes, got {}",
                self.len()
            ))
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
    fn peek_const<const N: usize>(&self) -> anyhow::Result<[u8; N]> {
        if self.len() < N {
            anyhow::bail!(format!(
                "Expected {N} remaining bytes, got {}",
                self.len()
            ))
        } else {
            let dst = &self[..N];
            // SAFETY: dst is guaranteed to be of length N
            // due to the slicing above which already implements bounds checks.
            unsafe { Ok(dst.try_into().unwrap_unchecked()) }
        }
    }
}

#[cfg(test)]
mod test {
    use paste::paste;

    use crate::bytes::{BinaryRead, BinaryWrite, BinVec};

    macro_rules! define_test_fns {
        ($($ty: ident),+) => {
            paste! {$(
                #[test]
                fn [<read_write_ $ty _ le>]() {
                    const VALUES: [$ty; 4] = [$ty::MAX, $ty::MIN, $ty::MAX - 42, $ty::MIN + 42];

                    let mut buffer = BinVec::new();
                    for v in VALUES {
                        buffer.[<write_ $ty _le>](v).unwrap();
                    }

                    let mut ss = buffer.as_slice();
                    for v in VALUES {
                        assert_eq!(v, ss.[<read_ $ty _le>]().unwrap());
                    }
                }

                #[test]
                fn [<read_write_ $ty _ be>]() {
                    const VALUES: [$ty; 4] = [$ty::MAX, $ty::MIN, $ty::MAX - 42, $ty::MIN + 42];

                    let mut buffer = BinVec::new();
                    for v in VALUES {
                        buffer.[<write_ $ty _be>](v).unwrap();
                    }

                    let mut ss = buffer.as_slice();
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

        let mut buffer = BinVec::new();
        for v in VALUES {
            buffer.write_u8(v).unwrap();
        }

        let mut ss = buffer.as_slice();
        for v in VALUES {
            assert_eq!(v, ss.read_u8().unwrap());
        }
    }

    #[test]
    fn read_write_i8() {
        const VALUES: [i8; 4] = [i8::MAX, i8::MIN, i8::MAX - 42, i8::MIN + 42];

        let mut buffer = BinVec::new();
        for v in VALUES {
            buffer.write_i8(v).unwrap();
        }

        let mut ss = buffer.as_slice();
        for v in VALUES {
            assert_eq!(v, ss.read_i8().unwrap());
        }
    }

    #[test]
    fn read_write_f32_le() {
        const VALUES: [f32; 4] =
            [f32::MAX, f32::MIN, f32::MAX - 42.0, f32::MIN + 42.0];

        let mut buffer = BinVec::new();
        for v in VALUES {
            buffer.write_f32_le(v).unwrap();
        }

        let mut ss = buffer.as_slice();
        for v in VALUES {
            assert_eq!(v, ss.read_f32_le().unwrap());
        }
    }

    #[test]
    fn read_write_f32_be() {
        const VALUES: [f32; 4] =
            [f32::MAX, f32::MIN, f32::MAX - 42.0, f32::MIN + 42.0];

        let mut buffer = BinVec::new();
        for v in VALUES {
            buffer.write_f32_be(v).unwrap();
        }

        let mut ss = buffer.as_slice();
        for v in VALUES {
            assert_eq!(v, ss.read_f32_be().unwrap());
        }
    }

    #[test]
    fn read_write_f64_le() {
        const VALUES: [f64; 4] =
            [f64::MAX, f64::MIN, f64::MAX - 42.0, f64::MIN + 42.0];

        let mut buffer = BinVec::new();
        for v in VALUES {
            buffer.write_f64_le(v).unwrap();
        }

        let mut ss = buffer.as_slice();
        for v in VALUES {
            assert_eq!(v, ss.read_f64_le().unwrap());
        }
    }

    #[test]
    fn read_write_f64_be() {
        const VALUES: [f64; 4] =
            [f64::MAX, f64::MIN, f64::MAX - 42.0, f64::MIN + 42.0];

        let mut buffer = BinVec::new();
        for v in VALUES {
            buffer.write_f64_be(v).unwrap();
        }

        let mut ss = buffer.as_slice();
        for v in VALUES {
            assert_eq!(v, ss.read_f64_be().unwrap());
        }
    }

    #[test]
    fn read_write_str() {
        let mut buffer = BinVec::new();
        buffer.write_str("Hello, World!").unwrap();

        let mut ss = buffer.as_slice();
        assert_eq!(ss.read_str().unwrap(), "Hello, World!");
    }
}
