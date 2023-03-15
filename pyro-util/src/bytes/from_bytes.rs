use crate::{u24::u24, Vector};
use std::mem;
use uuid::Uuid;
use crate::Result;
use paste::paste;

macro_rules! declare_read_fns {
    ($($ty: ident),+) => {
        paste! {$(
            #[doc = concat!("Reads a little endian [`", stringify!($ty), "`] from the buffer")]
            fn [<read_ $ty _le>] (&mut self) -> $crate::Result<$ty>;
            #[doc = concat!("Reads a big endian [`", stringify!($ty), "`] from the buffer")]
            fn [<read_ $ty _be>] (&mut self) -> $crate::Result<$ty>;
        )+}
    }
}

pub trait BinRead {
    fn take_n(&mut self, n: usize) -> Result<&[u8]>;
    fn take_const<const N: usize>(&mut self) -> Result<[u8; N]>;
    fn peek(&self, n: usize) -> Result<&[u8]>;
    fn peek_const<const N: usize>(&self) -> Result<[u8; N]>;

    fn read_bool(&mut self) -> Result<bool>;
    fn read_u8(&mut self) -> Result<u8>;
    fn read_i8(&mut self) -> Result<i8>;

    declare_read_fns!(u16, i16, u32, i32, u64, i64, u128, i128, f32, f64);

    fn read_var_u32(&mut self) -> Result<u32>;
    fn read_var_u64(&mut self) -> Result<u64>;
    fn read_var_i32(&mut self) -> Result<i32>;
    fn read_var_i64(&mut self) -> Result<i64>;

    fn read_u16_str(&mut self) -> Result<&str>;
}