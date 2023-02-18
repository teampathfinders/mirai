use bytes::{Buf, BytesMut};

use common::nvassert;
use common::Deserialize;
use common::VResult;

/// Sent by the client to initiate a full connection.
/// [`ConnectionRequestAccepted`](super::connection_request_accepted::ConnectionRequestAccepted) should be sent in response.
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

impl Deserialize for ConnectionRequest {
    fn deserialize(mut buffer: BytesMut) -> VResult<Self> {
        nvassert!(buffer.get_u8() == Self::ID);

        let guid = buffer.get_i64();
        let time = buffer.get_i64();

        Ok(Self { guid, time })
    }
}
