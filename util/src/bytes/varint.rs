use std::ops::ShrAssign;

use num_traits::FromPrimitive;

/// Trait implemented for types that can be used as variable integers.
pub trait VarInt:
    Sized + FromPrimitive + ShrAssign<Self> + PartialOrd<Self>
{
    fn var_len(self) -> usize {
        size_of_varint(self)
    }
}

impl VarInt for u32 {}

impl VarInt for i32 {}

impl VarInt for u64 {}

impl VarInt for i64 {}

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
