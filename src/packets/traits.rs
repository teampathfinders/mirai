use bytes::BytesMut;

use crate::error::VexResult;

/// Trait that all server to client game packets should implement.
pub trait GameEncodable {
    /// Encodes the packet into proper binary format.
    fn encode(&self, buffer: &mut BytesMut) -> VexResult<()>;
}

/// Trait that all client to server game packets should implement.
pub trait GameDecodable {
    /// Decodes the buffer into the specified packet.
    fn decode(buffer: BytesMut) -> VexResult<Self>
        where
            Self: Sized;
}

pub trait GamePacket {
    const ID: u32;
}