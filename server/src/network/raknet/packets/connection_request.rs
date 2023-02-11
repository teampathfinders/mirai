use bytes::{Buf, BytesMut};

use crate::network::traits::Decodable;
use common::vassert;
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

impl Decodable for ConnectionRequest {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        vassert!(buffer.get_u8() == Self::ID);

        let guid = buffer.get_i64();
        let time = buffer.get_i64();

        Ok(Self { guid, time })
    }
}
