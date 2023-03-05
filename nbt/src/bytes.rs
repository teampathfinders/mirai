use std::{cmp, fmt, io};
use std::fmt::Debug;
use std::io::Read;
use std::ops::{Deref, Index};
use crate::Buf;

pub struct Buffer<'a>(&'a [u8]);

impl<'a> From<&'a [u8]> for Buffer<'a> {
    #[inline]
    fn from(b: &'a [u8]) -> Self {
        Self(b)
    }
}

impl<'a> Deref for Buffer<'a> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> Index<usize> for Buffer<'a> {
    type Output = u8;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<'a> IntoIterator for Buffer<'a> {
    type Item = &'a u8;
    type IntoIter = std::slice::Iter<'a, u8>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> Debug for Buffer<'a> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self.0)
    }
}

impl<'a> Read for Buffer<'a> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let amt = cmp::min(self.len(), buf.len());
        let (a, b) = self.0.split_at(amt);

        if amt == 1 {
            buf[0] = a[0];
        } else {
            buf[..amt].copy_from_slice(a);
        }

        *self = Buffer::from(b);
        Ok(amt)
    }
}

impl<'a> Buf for Buffer<'a> {
    type Error = io::Error;

    fn read_u8(&mut self) -> io::Result<u8> {
        todo!()
    }

    fn read_u16(&mut self) -> io::Result<u16> {
        todo!()
    }

    fn read_u32(&mut self) -> io::Result<u32> {
        todo!()
    }

    fn read_u64(&mut self) -> io::Result<u64> {
        todo!()
    }

    fn read_u128(&mut self) -> io::Result<u128> {
        todo!()
    }

    fn read_i8(&mut self) -> io::Result<i8> {
        todo!()
    }

    fn read_i16(&mut self) -> io::Result<i16> {
        todo!()
    }

    fn read_i32(&mut self) -> io::Result<i32> {
        todo!()
    }

    fn read_i64(&mut self) -> io::Result<i64> {
        todo!()
    }

    fn read_i128(&mut self) -> io::Result<i128> {
        todo!()
    }

    fn read_u8_le(&mut self) -> io::Result<u8> {
        todo!()
    }

    fn read_u16_le(&mut self) -> io::Result<u16> {
        todo!()
    }

    fn read_u32_le(&mut self) -> io::Result<u32> {
        todo!()
    }

    fn read_u64_le(&mut self) -> io::Result<u64> {
        todo!()
    }

    fn read_u128_le(&mut self) -> io::Result<u128> {
        todo!()
    }

    fn read_i8_le(&mut self) -> io::Result<i8> {
        todo!()
    }

    fn read_i16_le(&mut self) -> io::Result<i16> {
        todo!()
    }

    fn read_i32_le(&mut self) -> io::Result<i32> {
        todo!()
    }

    fn read_i64_le(&mut self) -> io::Result<i64> {
        todo!()
    }

    fn read_i128_le(&mut self) -> io::Result<i128> {
        todo!()
    }
}