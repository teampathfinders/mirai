use util::bytes::{BinaryRead, SharedBuffer};
use util::Deserialize;
use util::pyassert;
use util::Result;

/// Sent by the client to initiate a full connection.
/// [`ConnectionRequestAccepted`](crate::connection_request_accepted::ConnectionRequestAccepted) should be sent in response.
#[derive(Debug)]
pub struct ConnectionRequest {
    /// Client-provided GUID.
    pub guid: i64,
    /// Timestamp of when this packet was sent.
    pub time: i64,
}

impl ConnectionRequest {
    /// Unique ID of this packet.
    pub const ID: u8 = 0x09;
}

impl Deserialize<'_> for ConnectionRequest {
    fn deserialize(mut buffer: SharedBuffer) -> Result<Self> {
        pyassert!(buffer.read_u8()? == Self::ID);

        let guid = buffer.read_i64_be()?;
        let time = buffer.read_i64_be()?;

        Ok(Self { guid, time })
    }
}
