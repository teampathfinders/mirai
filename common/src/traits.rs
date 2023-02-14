use bytes::BytesMut;

use crate::VResult;

/// Trait that all server to client packets should implement.
/// The Clone trait is required for broadcasting
pub trait Encodable: Clone {
    /// Encodes the packet into proper binary format.
    fn encode(&self) -> VResult<BytesMut>;
}

/// Trait that all client to server packets should implement.
pub trait Decodable {
    /// Decodes the buffer into the specified packet.
    fn decode(buffer: BytesMut) -> VResult<Self>
    where
        Self: Sized;
}
