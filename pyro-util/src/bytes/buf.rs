use std::{fmt, mem};
use std::mem::MaybeUninit;
use crate::{Vector, u24::u24};

pub trait FromBytes: Sized {
    const SIZE: usize;

    fn from_bytes_le(bytes: [u8; Self::SIZE]) -> Self;
    fn from_bytes_be(bytes: [u8; Self::SIZE]) -> Self;
}

macro_rules! from_bytes {
    ($t: ty) => {
        from_bytes!($t, <$t>::BITS);
    };

    ($t: ty, $b: expr) => {
        impl FromBytes for $t {
            const SIZE: usize = $b as usize / 8;

            #[inline]
            fn from_bytes_le(bytes: [u8; Self::SIZE]) -> Self {
                <$t>::from_le_bytes(bytes)
            }

            #[inline]
            fn from_bytes_be(bytes: [u8; Self::SIZE]) -> Self {
                <$t>::from_be_bytes(bytes)
            }
        }
    };
}

from_bytes!(u8);
from_bytes!(u16);
from_bytes!(u24);
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
    fn from_bytes_le(bytes: [u8; Self::SIZE]) -> Self {
        bytes[0] != 0
    }

    #[inline]
    fn from_bytes_be(bytes: [u8; Self::SIZE]) -> Self {
        bytes[0] != 0
    }
}

impl<T: FromBytes, const N: usize> FromBytes for [T; N] {
    const SIZE: usize = N * T::SIZE;

    #[inline]
    fn from_bytes_le(mut bytes: [u8; Self::SIZE]) -> Self {
        // Reverse bytes if the current machine is big endian.
        if cfg!(target_endian = "big") {
            for i in 0..N {
                (&mut [(i * T::SIZE)..((i + 1) * T::SIZE)]).reverse();
            }
        }

        // SAFETY: This is safe because Self::SIZE is guaranteed to be equal to T::SIZE * N.
        // A transmute_copy is required because the compiler
        // can't prove that both types are of the same size.
        let cast = unsafe {
            mem::transmute_copy::<[u8; Self::SIZE], [T; N]>(&bytes)
        };

        mem::forget(bytes);
        cast
    }
    
    #[inline]
    fn from_bytes_be(bytes: [u8; Self::SIZE]) -> Self {
        // Reverse bytes if the current machine is little endian.
        if cfg!(target_endian = "little") {
            for i in 0..N {
                (&mut [(i * T::SIZE)..((i + 1) * T::SIZE)]).reverse();
            }
        }

        // SAFETY: This is safe because Self::SIZE is guaranteed to be equal to T::SIZE * N.
        // A transmute_copy is required because the compiler
        // can't prove that both types are of the same size.
        let cast = unsafe {
            mem::transmute_copy::<[u8; Self::SIZE], [T; N]>(&bytes)
        };

        mem::forget(bytes);
        cast
    }
}

impl<T: FromBytes, const N: usize> FromBytes for Vector<T, N> {
    const SIZE: usize = N * std::mem::size_of::<T>();

    #[inline]
    fn from_bytes_le(bytes: [u8; Self::SIZE]) -> Self {
        // Reverse bytes if the current machine is little endian.
        if cfg!(target_endian = "big") {
            for i in 0..N {
                (&mut [(i * T::SIZE)..((i + 1) * T::SIZE)]).reverse();
            }
        }

        // SAFETY: This is safe because Self::SIZE is guaranteed to be equal to T::SIZE * N.
        // A transmute_copy is required because the compiler
        // can't prove that both types are of the same size.
        let cast = unsafe {
            mem::transmute_copy::<[u8; Self::SIZE], [T; N]>(&bytes)
        };

        mem::forget(bytes);
        Vector::from(cast)
    }

    #[inline]
    fn from_bytes_be(bytes: [u8; Self::SIZE]) -> Self {
        // Reverse bytes if the current machine is little endian.
        if cfg!(target_endian = "little") {
            for i in 0..N {
                (&mut [(i * T::SIZE)..((i + 1) * T::SIZE)]).reverse();
            }
        }

        // SAFETY: This is safe because Self::SIZE is guaranteed to be equal to T::SIZE * N.
        // A transmute_copy is required because the compiler
        // can't prove that both types are of the same size.
        let cast = unsafe {
            mem::transmute_copy::<[u8; Self::SIZE], [T; N]>(&bytes)
        };

        mem::forget(bytes);
        Vector::from(cast)
    }
}