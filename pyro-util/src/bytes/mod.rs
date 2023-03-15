use crate::Result;

mod from_bytes;
mod lazy;
mod shared;
mod to_bytes;
mod varint;

pub use from_bytes::*;
pub use lazy::*;
pub use shared::*;
pub use to_bytes::*;
pub use varint::*;

pub trait BinaryBuffer {
    fn take(&mut self, n: usize) -> Result<&[u8]>;
    fn take_const<const N: usize>(&mut self) -> Result<[u8; N]>;

    fn peek(&self, n: usize) -> Result<&[u8]>;
    fn peek_const<const N: usize>(&self) -> Result<[u8; N]>;

    fn read_le<T>(&mut self) -> Result<T>
    where
        T: FromBytes,
        [(); T::SIZE]:;

    fn read_be<T>(&mut self) -> Result<T>
    where
        T: FromBytes,
        [(); T::SIZE]:;

    fn read_var<T>(&mut self) -> Result<T>
    where
        T: VarInt;
}
