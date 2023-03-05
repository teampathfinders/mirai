#![warn(clippy::nursery)]

mod bytes;
mod de;
mod error;
mod ser;

pub use error::{Error, Result};