#![warn(clippy::nursery)]

use std::collections::HashMap;
use thiserror::Error;

#[cfg(test)]
mod test;

mod read_be;
mod read_le;

mod write_be;
mod write_le;
mod write_net;

pub use read_be::*;
pub use read_le::*;

pub use write_be::*;
pub use write_le::*;
pub use write_net::*;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid TAG type: {0}")]
    InvalidTag(u8),
}

pub const TAG_END: u8 = 0x00;
pub const TAG_BYTE: u8 = 0x01;
pub const TAG_SHORT: u8 = 0x02;
pub const TAG_INT: u8 = 0x03;
pub const TAG_LONG: u8 = 0x04;
pub const TAG_FLOAT: u8 = 0x05;
pub const TAG_DOUBLE: u8 = 0x06;
pub const TAG_BYTE_ARRAY: u8 = 0x07;
pub const TAG_STRING: u8 = 0x08;
pub const TAG_LIST: u8 = 0x09;
pub const TAG_COMPOUND: u8 = 0x0a;
pub const TAG_INT_ARRAY: u8 = 0x0b;
pub const TAG_LONG_ARRAY: u8 = 0x0c;

/// NBT tag value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    List(Vec<Value>),
    Compound(HashMap<String, Value>),
    ByteArray(Vec<i8>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl Value {
    /// Converts the value type to a numeric ID.
    pub const fn as_numeric_id(&self) -> u8 {
        match self {
            Self::End => TAG_END,
            Self::Byte(_) => TAG_BYTE,
            Self::Short(_) => TAG_SHORT,
            Self::Int(_) => TAG_INT,
            Self::Long(_) => TAG_LONG,
            Self::Float(_) => TAG_FLOAT,
            Self::Double(_) => TAG_DOUBLE,
            Self::String(_) => TAG_STRING,
            Self::List(_) => TAG_LIST,
            Self::Compound(_) => TAG_COMPOUND,
            Self::ByteArray(_) => TAG_BYTE_ARRAY,
            Self::IntArray(_) => TAG_INT_ARRAY,
            Self::LongArray(_) => TAG_LONG_ARRAY,
        }
    }
}

/// An NBT tag.
#[derive(Debug, Clone, PartialEq)]
pub struct RefTag<'a> {
    /// Name of this tag.
    pub name: &'a str,
    /// Value of this tag.
    pub value: &'a Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    pub name: String,
    pub value: Value,
}
