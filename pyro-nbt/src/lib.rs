mod de;
mod ser;

pub struct LittleEndian;

impl NbtFlavor for LittleEndian {
    const AS_ENUM: Flavor = Flavor::Little;
}

pub struct BigEndian;

impl NbtFlavor for BigEndian {
    const AS_ENUM: Flavor = Flavor::Big;
}

pub struct VarEndian;

impl NbtFlavor for VarEndian {
    const AS_ENUM: Flavor = Flavor::Var;
}

pub trait NbtFlavor {
    const AS_ENUM: Flavor;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Flavor {
    Little,
    Big,
    Var,
}

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
    LongArray,
}

impl TryFrom<u8> for FieldType {
    type Error = util::Error;

    fn try_from(v: u8) -> util::Result<Self> {
        const LAST_DISC: u8 = FieldType::LongArray as u8;
        if v > LAST_DISC {
            util::bail!(Other, "NBT field type discriminant out of range");
        }

        // SAFETY: Because `Self` is marked as `repr(u8)`, its layout is guaranteed to start
        // with a `u8` discriminant as its first field. Additionally, the raw discriminant is verified
        // to be in the enum's range.
        Ok(unsafe { std::mem::transmute::<u8, FieldType>(v) })
    }
}

pub use crate::de::{
    from_be_bytes, from_le_bytes, from_var_bytes, Deserializer,
};
pub use crate::ser::Serializer;
