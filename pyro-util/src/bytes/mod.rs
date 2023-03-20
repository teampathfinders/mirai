pub use mutable::*;
pub use read::*;
pub use shared::*;
pub use varint::*;
pub use write::*;

mod mutable;
mod read;
mod shared;
mod varint;
mod write;

/// Size of an IPv4 address in bytes.
pub const IPV4_MEM_SIZE: usize = 1 + 4 + 2;
/// Size of an IPv6 address in bytes.
pub const IPV6_MEM_SIZE: usize = 1 + 2 + 2 + 4 + 16 + 4;

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
