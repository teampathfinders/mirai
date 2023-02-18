use bytes::BytesMut;

use crate::VResult;

/// Trait that describes an object that can be serialised from raw bytes.
pub trait Serialize {
    /// Serializes the object into binary format.
    fn serialize(&self) -> VResult<BytesMut>;
}

/// Trait that describes an object that can be deserialised from raw bytes.
pub trait Deserialize {
    /// Deserializes the given buffer, returning the object.
    fn deserialize(buffer: BytesMut) -> VResult<Self>
    where
        Self: Sized;
}
