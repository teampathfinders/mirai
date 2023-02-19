use bytes::Bytes;
use bytes::{Buf, BytesMut};

use common::nvassert;
use common::Deserialize;
use common::ReadExtensions;
use common::VResult;

/// Sent by the client, in response to [`OpenConnectionReply2`](super::open_connection_reply2::OpenConnectionReply2).
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

impl Deserialize for OpenConnectionRequest2 {
    fn deserialize(mut buffer: Bytes) -> VResult<Self> {
        nvassert!(buffer.get_u8() == Self::ID);

        buffer.advance(16); // Skip magic
        buffer.get_addr()?; // Skip server address
        let mtu = buffer.get_u16();
        let client_guid = buffer.get_u64();

        Ok(Self { mtu, client_guid })
    }
}
