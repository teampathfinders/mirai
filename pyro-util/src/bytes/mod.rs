use crate::Result;

mod mutable;
mod read;
mod shared;
mod varint;
mod write;

pub use mutable::*;
pub use read::*;
pub use shared::*;
pub use varint::*;
pub use write::*;

// pub trait ReadableBuffer {
//     fn take(&mut self, n: usize) -> Result<&[u8]>;
//     fn take_const<const N: usize>(&mut self) -> Result<[u8; N]>;
//
//     fn peek(&self, n: usize) -> Result<&[u8]>;
//     fn peek_const<const N: usize>(&self) -> Result<[u8; N]>;
//
//     fn read_le<T>(&mut self) -> Result<T>
//     where
//         T: FromBytes,
//         [(); T::SIZE]:;
//
//     fn read_be<T>(&mut self) -> Result<T>
//     where
//         T: FromBytes,
//         [(); T::SIZE]:;
//
//     fn read_var<T>(&mut self) -> Result<T>
//     where
//         T: VarInt;
// }
