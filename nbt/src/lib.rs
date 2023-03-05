#![warn(clippy::nursery)]

mod de;
mod error;
mod ser;

pub use error::{Error, Result};