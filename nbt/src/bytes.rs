use crate::{Buf, Error};
use crate::buf::FromBytes;
use std::fmt::Debug;
use std::io::Read;
use std::ops::{Deref, Index};
use std::{cmp, fmt, io};

use crate::Result;

pub struct ReadBuffer<'a>(&'a [u8]);

impl<'a> ReadBuffer<'a> {
    #[inline]
    pub fn advance(&mut self, n: usize) {
        let (_, b) = self.0.split_at(n);
        *self = ReadBuffer::from(b);
    }

    #[inline]
    pub fn peek<T: FromBytes>(&self) -> Result<T> {
        todo!();
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn take_n(&mut self, n: usize) -> Result<&[u8]> {
        if self.len() < n {
            Err(Error::UnexpectedEof)
        } else {
            let (a, b) = self.0.split_at(n);
            *self = ReadBuffer::from(b);
            Ok(a)
        }
    }

    #[inline]
    pub fn take<const N: usize>(&mut self) -> Result<[u8; N]> {
        if self.len() < N {
            Err(Error::UnexpectedEof)
        } else {
            let (a, b) = self.0.split_at(N);
            *self = ReadBuffer::from(b);
            Ok(a.try_into().unwrap())
        }
    }

    #[inline]
    pub fn first(&self) -> Result<&u8> {
        self.0.first().ok_or(Error::UnexpectedEof)
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

impl<'a> IntoIterator for ReadBuffer<'a> {
    type Item = &'a u8;
    type IntoIter = std::slice::Iter<'a, u8>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

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

impl<'a> Buf for ReadBuffer<'a> {
    #[inline]
    fn read_bool(&mut self) -> Result<bool> {
        let x = self.read_u8()?;
        Ok(x != 0)
    }

    #[inline]
    fn read_u8(&mut self) -> Result<u8> {
        let x = self.first().copied()?;
        self.advance(1);
        Ok(x)
    }

    #[inline]
    fn read_u16(&mut self) -> Result<u16> {
        let b = self.take()?;
        Ok(u16::from_be_bytes(b))
    }

    #[inline]
    fn read_u32(&mut self) -> Result<u32> {
        let b = self.take()?;
        Ok(u32::from_be_bytes(b))
    }

    #[inline]
    fn read_u64(&mut self) -> Result<u64> {
        let b = self.take()?;
        Ok(u64::from_be_bytes(b))
    }

    #[inline]
    fn read_u128(&mut self) -> Result<u128> {
        let b = self.take()?;
        Ok(u128::from_be_bytes(b))
    }

    #[inline]
    fn read_i8(&mut self) -> Result<i8> {
        let x = self.first().copied()? as i8;
        self.advance(1);
        Ok(x)
    }

    #[inline]
    fn read_i16(&mut self) -> Result<i16> {
        let b = self.take()?;
        Ok(i16::from_be_bytes(b))
    }

    #[inline]
    fn read_i32(&mut self) -> Result<i32> {
        let b = self.take()?;
        Ok(i32::from_be_bytes(b))
    }

    #[inline]
    fn read_i64(&mut self) -> Result<i64> {
        let b = self.take()?;
        Ok(i64::from_be_bytes(b))
    }

    #[inline]
    fn read_i128(&mut self) -> Result<i128> {
        let b = self.take()?;
        Ok(i128::from_be_bytes(b))
    }

    #[inline]
    fn read_u16_le(&mut self) -> Result<u16> {
        let b = self.take()?;
        Ok(u16::from_le_bytes(b))
    }

    #[inline]
    fn read_u32_le(&mut self) -> Result<u32> {
        let b = self.take()?;
        Ok(u32::from_le_bytes(b))
    }

    #[inline]
    fn read_u64_le(&mut self) -> Result<u64> {
        let b = self.take()?;
        Ok(u64::from_le_bytes(b))
    }

    #[inline]
    fn read_u128_le(&mut self) -> Result<u128> {
        let b = self.take()?;
        Ok(u128::from_le_bytes(b))
    }

    #[inline]
    fn read_i16_le(&mut self) -> Result<i16> {
        let b = self.take()?;
        Ok(i16::from_le_bytes(b))
    }

    #[inline]
    fn read_i32_le(&mut self) -> Result<i32> {
        let b = self.take()?;
        Ok(i32::from_le_bytes(b))
    }

    #[inline]
    fn read_i64_le(&mut self) -> Result<i64> {
        let b = self.take()?;
        Ok(i64::from_le_bytes(b))
    }

    #[inline]
    fn read_i128_le(&mut self) -> Result<i128> {
        let b = self.take()?;
        Ok(i128::from_le_bytes(b))
    }

    #[inline]
    fn read_f32(&mut self) -> Result<f32> {
        let b = self.take()?;
        Ok(f32::from_be_bytes(b))
    }

    #[inline]
    fn read_f32_le(&mut self) -> Result<f32> {
        let b = self.take()?;
        Ok(f32::from_le_bytes(b))
    }

    #[inline]
    fn read_f64(&mut self) -> Result<f64> {
        let b = self.take()?;
        Ok(f64::from_be_bytes(b))
    }

    #[inline]
    fn read_f64_le(&mut self) -> Result<f64> {
        let b = self.take()?;
        Ok(f64::from_le_bytes(b))
    }
}

#[cfg(test)]
mod test {
    use super::{Buf, ReadBuffer};

    #[test]
    fn test_read_u8() {
        let s: &[u8] = &[42, 12, 1, 2, 3];
        let mut buf = ReadBuffer::from(s);

        for x in s {
            assert_eq!(buf.read_u8().unwrap(), *x);
        }
    }

    #[test]
    fn test_read_i8() {
        let s: &[i8] = &[-10, 5, -42, 120];
        let mut buf =
            ReadBuffer::from(unsafe { std::mem::transmute::<&[i8], &[u8]>(s) });

        for x in s {
            assert_eq!(buf.read_i8().unwrap(), *x);
        }
    }

    #[test]
    fn test_read_u16() {
        let o = [42, 24083];
        let s: &[u8] = &[0, 42, 94, 19];
        let mut buf = ReadBuffer::from(s);

        for x in o {
            assert_eq!(buf.read_u16().unwrap(), x);
        }
    }

    #[test]
    fn test_read_i16() {
        let o = [-2397, 24083];
        let s: &[u8] = &[246, 163, 94, 19];
        let mut buf = ReadBuffer::from(s);

        for x in o {
            assert_eq!(buf.read_i16().unwrap(), x);
        }
    }
}
