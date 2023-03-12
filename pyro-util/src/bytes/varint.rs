use std::ops::ShrAssign;
use num_traits::FromPrimitive;
use crate::bytes::ReadBuffer;

use crate::{bail, Result};

/// Trait implemented for types that can be used as variable integers.
pub trait VarInt: Sized + FromPrimitive + ShrAssign<Self> + PartialOrd<Self>
{
    fn var_len(self) -> usize {
        size_of_varint(self)
    }

    fn read(buf: &mut ReadBuffer) -> Result<Self>;
}

impl VarInt for u32 {
    fn read(buf: &mut ReadBuffer) -> Result<Self> {
        let mut v = 0;
        let mut i = 0;
        while i < 35 {
            let b = buf.read_le::<u8>()?;
            v |= ((b & 0x7f) as u32) << i;
            if b & 0x80 == 0 {
                return Ok(v);
            }
            i += 7;
        }

        bail!(
            Malformed,
            "Variable 32-bit integer did not end after 5 bytes"
        )
    }
}

impl VarInt for i32 {
    fn read(buf: &mut ReadBuffer) -> Result<Self> {
        let vx = buf.read_var::<u32>()?;
        let mut v = (vx >> 1) as i32;

        if vx & 1 != 0 {
            v = !v;
        }

        Ok(v)
    }
}

impl VarInt for u64 {
    fn read(buf: &mut ReadBuffer) -> Result<Self> {
        let mut v = 0;
        let mut i = 0;
        while i < 70 {
            let b = buf.read_le::<u8>()?;
            v |= ((b & 0x7f) as u64) << i;

            if b & 0x80 == 0 {
                return Ok(v);
            }

            i += 7;
        }

        bail!(
            Malformed,
            "Variable 64-bit integer did not end after 10 bytes"
        )
    }
}

impl VarInt for i64 {
    fn read(buf: &mut ReadBuffer) -> Result<Self> {
        let vx = buf.read_var::<u64>()?;
        let mut v = (vx >> 1) as i64;

        if vx & 1 != 0 {
            v = !v;
        }

        Ok(v)
    }
}

pub fn size_of_string(value: &str) -> usize {
    size_of_varint(value.len() as u32) + value.len()
}

/// Determines the size in bytes of the given variable integer.
pub fn size_of_varint<T: VarInt + FromPrimitive>(mut value: T) -> usize {
    let mut count = 0;
    while value >= T::from_u32(0x80).unwrap() {
        count += 1;
        value >>= T::from_u32(7).unwrap();
    }

    count + 1
}

pub trait VarString {
    fn var_len(&self) -> usize;
}

impl VarString for &str {
    fn var_len(&self) -> usize {
        size_of_varint(self.len() as u32) + self.len()
    }
}

impl VarString for String {
    fn var_len(&self) -> usize {
        size_of_varint(self.len() as u32) + self.len()
    }
}