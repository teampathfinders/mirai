use crate::{bytes::{MutableBuffer, SharedBuffer}, BinaryRead};
use std::fmt::Debug;

/// Trait that describes an object that can be serialised from raw bytes.
pub trait Serialize {
    /// Serializes the object into binary format.
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()>;
}

/// Trait that describes an object that can be deserialised from raw bytes.
pub trait Deserialize<'a>: Sized {
    /// Deserializes the given buffer, returning the object.
    fn deserialize<R: BinaryRead<'a>>(reader: R) -> anyhow::Result<Self>;
    // fn deserialize(buffer: SharedBuffer<'a>) -> anyhow::Result<Self>;


    /// Deserializes the given buffer, returning the object.
    /// While [`deserialize`](Self::deserialize) consumes the buffer, this function
    /// modifies the original buffer allowing you to continue where this function left off.
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self>;
    // fn deserialize_from(buffer: &mut SharedBuffer<'a>) -> anyhow::Result<Self>;
}

/// Adds the [`try_expect`](TryExpect::try_expect) function to an object.
pub trait TryExpect {
    /// Output type on successful call.
    type Output;

    /// Similar to the built-in expect function but instead of panicking, it returns an error.
    fn try_expect<E: Debug>(self, message: E) -> anyhow::Result<Self::Output>;
}

impl<T> TryExpect for Option<T> {
    type Output = T;

    fn try_expect<E: Debug>(self, error: E) -> anyhow::Result<Self::Output> {
        match self {
            Some(s) => Ok(s),
            None => anyhow::bail!(format!("{error:?}")),
        }
    }
}
