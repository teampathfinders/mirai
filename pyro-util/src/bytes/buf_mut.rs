use crate::u24::u24;

pub trait ToBytes: Sized {
    const SIZE: usize;

    fn to_bytes_le(self) -> [u8; Self::SIZE];
    fn to_bytes_be(self) -> [u8; Self::SIZE];
}

macro_rules! to_bytes {
    ($t: ty) => {
        to_bytes!($t, <$t>::BITS);
    };

    ($t: ty, $b: expr) => {
        impl ToBytes for $t {
            const SIZE: usize = $b as usize / 8;

            #[inline]
            fn to_bytes_le(self) -> [u8; Self::SIZE] {
                self.to_le_bytes()
            }

            #[inline]
            fn to_bytes_be(self) -> [u8; Self::SIZE] {
                self.to_be_bytes()
            }
        }
    };
}

to_bytes!(u8);
to_bytes!(u16);
to_bytes!(u24);
to_bytes!(u32);
to_bytes!(u64);
to_bytes!(u128);
to_bytes!(i8);
to_bytes!(i16);
to_bytes!(i32);
to_bytes!(i64);
to_bytes!(i128);
to_bytes!(f32, 32); // f32 does not have a BITS associated constant
to_bytes!(f64, 64); // f64 does not have a BITS associated constant

impl ToBytes for bool {
    const SIZE: usize = 1;

    #[inline]
    fn to_bytes_le(self) -> [u8; Self::SIZE] {
        [self as u8]
    }

    #[inline]
    fn to_bytes_be(self) -> [u8; Self::SIZE] {
        [self as u8]
    }
}

