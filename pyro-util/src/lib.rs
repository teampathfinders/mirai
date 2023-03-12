#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

/// Shorthand for `mod module; pub use module::*;`.
#[macro_export]
macro_rules! glob_export {
    ($module: ident) => {
        mod $module;
        pub use $module::*;
    };
}

#[cfg(test)]
mod test;

pub mod bytes;
mod error;
mod varint;
mod extensions;
mod traits;
mod vector;
mod u24;

pub use u24::*;
pub use error::*;
pub use varint::*;
pub use extensions::*;
pub use traits::*;
pub use vector::*;
