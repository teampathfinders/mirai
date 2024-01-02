use crate::glob_export;

glob_export!(mutable);
glob_export!(read);
glob_export!(shared);
glob_export!(varint);
glob_export!(write);

/// Size of an IPv4 address in bytes.
pub const IPV4_MEM_SIZE: usize = 1 + 4 + 2;
/// Size of an IPv6 address in bytes.
pub const IPV6_MEM_SIZE: usize = 1 + 2 + 2 + 4 + 16 + 4;
