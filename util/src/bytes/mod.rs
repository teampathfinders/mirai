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
