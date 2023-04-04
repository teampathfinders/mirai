pub use key::*;
pub use level::*;
pub use sub_chunk::*;

/// Performs ceiling division on two u32s.
#[inline]
const fn ceil_div(lhs: u32, rhs: u32) -> u32 {
    (lhs + rhs - 1) / rhs
}

#[cfg(test)]
mod test;

mod biome;
pub mod database;
mod ffi;
mod key;
mod level;
mod level_dat;
mod sub_chunk;
