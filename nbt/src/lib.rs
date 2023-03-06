#![feature(min_specialization)]
#![warn(clippy::nursery)]

mod buf_mut;
mod buf;
mod bytes_mut;
mod bytes;
mod de;
mod error;
mod ser;

const TAG_END: u8 = 0x00;
const TAG_BYTE: u8 = 0x01;
const TAG_SHORT: u8 = 0x02;
const TAG_INT: u8 = 0x03;
const TAG_LONG: u8 = 0x04;
const TAG_FLOAT: u8 = 0x05;
const TAG_DOUBLE: u8 = 0x06;
const TAG_BYTE_ARRAY: u8 = 0x07;
const TAG_STRING: u8 = 0x08;
const TAG_LIST: u8 = 0x09;
const TAG_COMPOUND: u8 = 0x0a;
const TAG_INT_ARRAY: u8 = 0x0b;
const TAG_LONG_ARRAY: u8 = 0x0c;

pub use crate::buf::Buf;
pub use crate::bytes::ReadBuffer;
pub use crate::error::{Error, Result};
