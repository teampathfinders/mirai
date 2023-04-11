use crate::bytes::{BinaryRead, BinaryWrite};

/// Trait that describes an object that can be serialised from raw bytes.
pub trait Serialize {
    /// Serializes the object into binary format.
    fn serialize<W>(&self, writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite;
}

/// Trait that describes an object that can be deserialised from raw bytes.
pub trait Deserialize<'a> {
    /// Deserializes the given buffer, returning the object.
    fn deserialize<R>(reader: R) -> anyhow::Result<Self>
    where
        R: BinaryRead<'a> + 'a,
        Self: Sized;
}
