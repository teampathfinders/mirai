#![warn(clippy::nursery)]

mod buf_mut;
mod buf;
mod bytes_mut;
mod bytes;
mod de;
mod error;
mod ser;

const TAG_END: u8 = 0x00;
const TAG_COMPOUND: u8 = 0x0a;

pub use crate::buf::Buf;
pub use crate::bytes::ReadBuffer;
pub use crate::error::{Error, Result};
