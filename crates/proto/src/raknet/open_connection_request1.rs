use util::iassert;
use util::{BinaryRead, Deserialize};

/// Sent by the client when the users joins the server.
#[derive(Debug)]
pub struct OpenConnectionRequest1 {
    /// Version of the Raknet protocol.
    /// Minecraft currently uses version 10.
    ///
    /// If this does not equal the server's version (['RAKNET_VERSION'](crate::raknet::RAKNET_VERSION)),
    /// then an [`IncompatibleProtocol`](crate::raknet::IncompatibleProtocol) packet should be sent.
    pub protocol_version: u8,
    /// Maximum Transfer Unit. Specifies the maximum size of raknet that the connection can handle.
    /// The client keeps sending raknet with continuously decreasing padding, until it receives a response.
    pub mtu: u16,
}

impl OpenConnectionRequest1 {
    /// Unique identifier for this packet.
    pub const ID: u8 = 0x05;
}

impl<'a> Deserialize<'a> for OpenConnectionRequest1 {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        iassert!(reader.read_u8()? == Self::ID);

        let mtu = reader.remaining() as u16 + 28;
        reader.advance(16)?; // Skip magic
        let protocol_version = reader.read_u8()?;

        Ok(Self { protocol_version, mtu })
    }
}
