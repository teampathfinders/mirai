#![feature(min_specialization)]
#![warn(clippy::nursery)]

mod de;
// mod ser;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
enum FieldType {
    End,
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    ByteArray,
    String,
    List,
    Compound,
    IntArray,
    LongArray
}

impl TryFrom<u8> for FieldType {
    type Error = Error;

    fn try_from(v: u8) -> Result<Self> {
        const LAST_DISC: u8 = FieldType::LongArray as u8;
        if v > LAST_DISC {
            bail!(Other, "NBT field type discriminant out of range");
        }

        // SAFETY: Because `Self` is marked as `repr(u8)`, its layout is guaranteed to start
        // with a `u8` discriminant as its first field. Additionally, the raw discriminant is verified
        // to be in the enum's range.
        Ok(unsafe {
            mem::transmute::<u8, FieldType>(v)
        })
    }
}

use std::mem;
use util::{bail, Error, Result};
pub use crate::de::{from_be_bytes, from_le_bytes, from_net_bytes};
