use util::bytes::{BinaryRead, SharedBuffer};
use util::pyassert;
use util::Deserialize;
use util::Result;

/// Sent by the client, in response to [`OpenConnectionReply2`](crate::raknet::OpenConnectionReply2).
#[derive(Debug)]
pub struct OpenConnectionRequest2 {
    /// MTU of the connection.
    pub mtu: u16,
    /// GUID of the client.
    pub client_guid: u64,
}

impl OpenConnectionRequest2 {
    /// Unique identifier of the packet.
    pub const ID: u8 = 0x07;
}

impl Deserialize<'_> for OpenConnectionRequest2 {
    fn deserialize(mut buffer: SharedBuffer) -> anyhow::Result<Self> {
        pyassert!(buffer.read_u8()? == Self::ID);

        buffer.advance(16); // Skip magic
        buffer.read_addr()?; // Skip server address
        let mtu = buffer.read_u16_be()?;
        let client_guid = buffer.read_u64_be()?;

        Ok(Self { mtu, client_guid })
    }
}
