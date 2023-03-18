mod de;
mod ser;

pub use crate::de::{
    from_be_bytes, from_le_bytes, from_var_bytes, Deserializer,
};
pub use crate::ser::Serializer;

mod private {
    use crate::{BigEndian, LittleEndian, Variable};

    /// Prevents [`VariantImpl`](super::VariantImpl) from being implemented for
    /// types outside of this crate.
    pub trait Sealed {}

    impl Sealed for LittleEndian {}
    impl Sealed for BigEndian {}
    impl Sealed for Variable {}
}

/// Implemented by all NBT variants.
pub trait VariantImpl: private::Sealed {
    /// Used to convert a variant to an enum.
    /// This is used to match generic types in order to prevent
    /// having to duplicate all deserialisation code three times.
    const AS_ENUM: Variant;
}

/// NBT format variant.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Variant {
    /// Used by Bedrock for data saved to disk.
    /// Every data type is written in little endian format.
    LittleEndian,
    /// Used by Java.
    /// Every data types is written in big endian format.
    BigEndian,
    /// Used by Bedrock for NBT transferred over the network.
    /// This format is the same as [`LittleEndian`], except that type lengths
    /// (such as for strings or lists), are varints instead of shorts.
    /// The integer and long types are also varints.
    Variable,
}

/// Used by Bedrock for data saved to disk.
/// Every data type is written in little endian format.
pub struct LittleEndian;

impl VariantImpl for LittleEndian {
    const AS_ENUM: Variant = Variant::LittleEndian;
}

/// Used by Java.
/// Every data types is written in big endian format.
pub struct BigEndian;

impl VariantImpl for BigEndian {
    const AS_ENUM: Variant = Variant::BigEndian;
}

/// Used by Bedrock for NBT transferred over the network.
/// This format is the same as [`LittleEndian`], except that type lengths
/// (such as for strings or lists), are varints instead of shorts.
/// The integer and long types are also varints.
pub struct Variable;

impl VariantImpl for Variable {
    const AS_ENUM: Variant = Variant::Variable;
}

/// NBT field type
// Compiler complains about unused enum variants even though they're constructed using a transmute.
#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
enum FieldType {
    /// Indicates the end of a compound tag.
    End,
    /// A signed byte.
    Byte,
    /// A signed short.
    Short,
    /// A signed int.
    Int,
    /// A signed long.
    Long,
    /// A float.
    Float,
    /// A double.
    Double,
    /// An array of byte tags.
    ByteArray,
    /// A UTF-8 string.
    String,
    /// List of tags.
    /// Every item in the list must be of the same type.
    List,
    /// A key-value map.
    Compound,
    /// An array of int tags.
    IntArray,
    /// An array of long tags.
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
