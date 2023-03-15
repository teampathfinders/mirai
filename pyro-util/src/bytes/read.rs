use crate::Result;
use crate::{u24::u24, Vector};
use paste::paste;
use std::mem;
use uuid::Uuid;

macro_rules! declare_read_fns {
    ($($ty: ident),+) => {
        paste! {$(
            #[doc = concat!("Reads a little endian [`", stringify!($ty), "`] from the reader")]
            fn [<read_ $ty _le>] (&mut self) -> $crate::Result<$ty>;
            #[doc = concat!("Reads a big endian [`", stringify!($ty), "`] from the reader")]
            fn [<read_ $ty _be>] (&mut self) -> $crate::Result<$ty>;
        )+}
    }
}

/// Adds binary reading capabilities to a reader.
pub trait BinaryReader {
    declare_read_fns!(u16, i16, u32, i32, u64, i64, u128, i128, f32, f64);

    /// Takes `n` bytes out of the reader.
    fn take_n(&mut self, n: usize) -> Result<&[u8]>;
    /// Takes `N` bytes out of the reader.
    /// This can be used to get sized arrays if the size is known at compile time.
    fn take_const<const N: usize>(&mut self) -> Result<[u8; N]>;
    /// Takes `n` bytes out of the reader without advancing the cursor.
    fn peek(&self, n: usize) -> Result<&[u8]>;
    /// Takes `N` bytes out of the reader without advancing the cursor.
    /// /// This can be used to get sized arrays if the size is known at compile time.
    fn peek_const<const N: usize>(&self) -> Result<[u8; N]>;
    /// Reads a [`bool`] from the reader.
    fn read_bool(&mut self) -> Result<bool>;
    /// Reads an [`u8`] from the reader.
    fn read_u8(&mut self) -> Result<u8>;
    /// Reads an [`i8`] from the reader.
    fn read_i8(&mut self) -> Result<i8>;
    /// Reads a variable size [`u32`] from the reader.
    fn read_var_u32(&mut self) -> Result<u32>;
    /// Reads a variable size [`i32`] from the reader.
    fn read_var_u64(&mut self) -> Result<u64>;
    /// Reads a variable size [`u64`] from the reader.
    fn read_var_i32(&mut self) -> Result<i32>;
    /// Reads a variable size [`i64`] from the reader.
    fn read_var_i64(&mut self) -> Result<i64>;
    /// Reads a string prefixed by a variable u32.
    fn read_str(&mut self) -> Result<&str>;
}
