use crate::error::VexResult;
use bytes::BytesMut;

/// Trait that all server to client packets should implement.
pub trait Encodable {
    /// Encodes the packet into proper binary format.
    fn encode(&self) -> VexResult<BytesMut>;
}

/// Trait that all client to server packets should implement.
pub trait Decodable {
    /// Decodes the buffer into the specified packet.
    fn decode(buffer: BytesMut) -> VexResult<Self>
    where
        Self: Sized;
}
