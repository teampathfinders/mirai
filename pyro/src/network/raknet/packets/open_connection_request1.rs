use bytes::Bytes;
use bytes::{Buf, BytesMut};

use util::nvassert;
use util::Deserialize;
use util::Result;

/// Sent by the client when the users joins the server.
#[derive(Debug)]
pub struct OpenConnectionRequest1 {
    /// Version of the Raknet protocol.
    /// Minecraft currently uses version 10.
    ///
    /// If this does not equal the server's version (['RAKNET_VERSION'](crate::network::raknet::RAKNET_VERSION)),
    /// then an [`IncompatibleProtocol`](super::incompatible_protocol::IncompatibleProtocol) packet should be sent.
    pub protocol_version: u8,
    /// Maximum Transfer Unit. Specifies the maximum size of packets that the connection can handle.
    /// The client keeps sending packets with continuously decreasing padding, until it receives a response.
    pub mtu: u16,
}

impl OpenConnectionRequest1 {
    /// Unique identifier for this packet.
    pub const ID: u8 = 0x05;
}

impl Deserialize for OpenConnectionRequest1 {
    fn deserialize(mut buffer: Bytes) -> Result<Self> {
        let mtu = buffer.len() as u16 + 28;

        nvassert!(buffer.get_u8() == Self::ID);

        buffer.advance(16); // Skip magic
        let protocol_version = buffer.get_u8();

        Ok(Self { protocol_version, mtu })
    }
}
