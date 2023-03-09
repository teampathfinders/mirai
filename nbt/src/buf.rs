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

            #[inline]
            fn from_le_bytes(bytes: [u8; Self::SIZE]) -> Self { <$t>::from_le_bytes(bytes) }

            #[inline]
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
from_bytes!(f32, 32); // f32 does not have a BITS associated constant
from_bytes!(f64, 64); // f64 does not have a BITS associated constant

impl FromBytes for bool {
    const SIZE: usize = 1;

    #[inline]
    fn from_le_bytes(bytes: [u8; Self::SIZE]) -> Self {
        bytes[0] != 0
    }

    #[inline]
    fn from_be_bytes(bytes: [u8; Self::SIZE]) -> Self {
        bytes[0] != 0
    }
}