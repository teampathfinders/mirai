#![warn(clippy::nursery)]

mod buf_mut;
mod buf;
mod bytes_mut;
mod bytes;
mod de;
mod error;
mod ser;

pub use crate::buf::Buf;
pub use crate::bytes::ReadBuffer;
pub use crate::error::{Error, Result};
