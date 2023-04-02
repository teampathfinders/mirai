use crate::bytes::{MutableBuffer, SharedBuffer};
use crate::Result;

/// Trait that describes an object that can be serialised from raw bytes.
pub trait Serialize {
    /// Serializes the object into binary format.
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()>;
}

/// Trait that describes an object that can be deserialised from raw bytes.
pub trait Deserialize<'a> {
    /// Deserializes the given buffer, returning the object.
    fn deserialize(buffer: SharedBuffer<'a>) -> anyhow::Result<Self>
    where
        Self: Sized;
}
