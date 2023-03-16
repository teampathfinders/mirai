use util::bytes::SharedBuffer;
use util::pyassert;
use util::Result;
use util::{bytes::BinaryReader, Deserialize};

/// Sent by the client when the users joins the server.
#[derive(Debug)]
pub struct OpenConnectionRequest1 {
    /// Version of the Raknet protocol.
    /// Minecraft currently uses version 10.
    ///
    /// If this does not equal the server's version (['RAKNET_VERSION'](crate::RAKNET_VERSION)),
    /// then an [`IncompatibleProtocol`](crate::incompatible_protocol::IncompatibleProtocol) packet should be sent.
    pub protocol_version: u8,
    /// Maximum Transfer Unit. Specifies the maximum size of packets that the connection can handle.
    /// The client keeps sending packets with continuously decreasing padding, until it receives a response.
    pub mtu: u16,
}

impl OpenConnectionRequest1 {
    /// Unique identifier for this packet.
    pub const ID: u8 = 0x05;
}

impl Deserialize<'_> for OpenConnectionRequest1 {
    fn deserialize(mut buffer: SharedBuffer) -> Result<Self> {
        let mtu = buffer.len() as u16 + 28;

        pyassert!(buffer.read_u8()? == Self::ID);

        buffer.advance(16); // Skip magic
        let protocol_version = buffer.read_u8()?;

        Ok(Self { protocol_version, mtu })
    }
}
