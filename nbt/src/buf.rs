use crate::Result;

pub trait FromBytes: Sized {
    const SIZE: usize;

    fn from_le_bytes(bytes: [u8; Self::SIZE]) -> Self;
    fn from_be_bytes(bytes: [u8; Self::SIZE]) -> Self;
}

macro_rules! from_bytes {
    ($t: ty) => {
        from_bytes!($t, <$t>::BITS);
    };

    ($t: ty, $b: expr) => {
        impl FromBytes for $t {
            const SIZE: usize = $b as usize / 8;

            fn from_le_bytes(bytes: [u8; Self::SIZE]) -> Self { <$t>::from_le_bytes(bytes) }
            fn from_be_bytes(bytes: [u8; Self::SIZE]) -> Self { <$t>::from_be_bytes(bytes) }
        }
    }
}

from_bytes!(u8);
from_bytes!(u16);
from_bytes!(u32);
from_bytes!(u64);
from_bytes!(u128);
from_bytes!(i8);
from_bytes!(i16);
from_bytes!(i32);
from_bytes!(i64);
from_bytes!(i128);
from_bytes!(f32, 32);
from_bytes!(f64, 64);

pub trait Buf {
    fn read_bool(&mut self) -> Result<bool>;

    fn read_u8(&mut self) -> Result<u8>;
    fn read_u16(&mut self) -> Result<u16>;
    fn read_u32(&mut self) -> Result<u32>;
    fn read_u64(&mut self) -> Result<u64>;
    fn read_u128(&mut self) -> Result<u128>;

    fn read_i8(&mut self) -> Result<i8>;
    fn read_i16(&mut self) -> Result<i16>;
    fn read_i32(&mut self) -> Result<i32>;
    fn read_i64(&mut self) -> Result<i64>;
    fn read_i128(&mut self) -> Result<i128>;

    fn read_u16_le(&mut self) -> Result<u16>;
    fn read_u32_le(&mut self) -> Result<u32>;
    fn read_u64_le(&mut self) -> Result<u64>;
    fn read_u128_le(&mut self) -> Result<u128>;

    fn read_i16_le(&mut self) -> Result<i16>;
    fn read_i32_le(&mut self) -> Result<i32>;
    fn read_i64_le(&mut self) -> Result<i64>;
    fn read_i128_le(&mut self) -> Result<i128>;

    fn read_f32(&mut self) -> Result<f32>;
    fn read_f32_le(&mut self) -> Result<f32>;
    fn read_f64(&mut self) -> Result<f64>;
    fn read_f64_le(&mut self) -> Result<f64>;
}
