use bytes::{Buf, BytesMut};

use crate::error::VexResult;
use crate::network::traits::Decodable;
use crate::vex_assert;

/// Sent by the client when the users joins the server.
#[derive(Debug)]
pub struct OpenConnectionRequest1 {
    /// Version of the Raknet protocol.
    /// Minecraft currently uses version 10.
    ///
    /// If this does not equal the server's version (['RAKNET_VERSION'](super::RAKNET_VERSION)),
    /// then an [`IncompatibleProtocol`](super::IncompatibleProtocol) packet should be sent.
    pub protocol_version: u8,
    /// Maximum Transfer Unit. Specifies the maximum size of packets that the connection can handle.
    /// The client keeps sending packets with continuously decreasing padding, until it receives a response.
    /// For most connections this value will be 1472.
    pub mtu: u16,
}

impl OpenConnectionRequest1 {
    /// Unique identifier for this packet.
    pub const ID: u8 = 0x05;
}

impl Decodable for OpenConnectionRequest1 {
    fn decode(mut buffer: BytesMut) -> VexResult<Self> {
        vex_assert!(buffer.get_u8() == Self::ID);

        buffer.advance(16); // Skip magic
        let protocol_version = buffer.get_u8();
        let mtu = buffer.len() as u16 - 18 + 46; // Size of padding + 46

        Ok(Self {
            protocol_version,
            mtu,
        })
    }
}
