#![warn(clippy::nursery)]

mod buf;
mod bytes;
mod de;
mod error;
mod ser;

pub use crate::error::{Error, Result};
pub use crate::bytes::Buffer;
pub use crate::buf::Buf;