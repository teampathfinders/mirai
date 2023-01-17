use crate::error::VexResult;
use crate::raknet::packets::{Decodable};
use crate::vex_assert;
use bytes::{Buf, BytesMut};

/// Sent to retrieve information about the server
/// while the user is in Minecraft's server tab.
/// An [`UnconnectedPong`](super::UnconnectedPong) packet should be sent in response.
#[derive(Debug)]
pub struct UnconnectedPing {
    /// Time when this ping was sent.
    /// Used to measure server latency.
    pub time: i64,
    /// GUID of the client.
    ///
    /// Unknown what this is used for.
    /// It's randomised each time Minecraft is restarted and therefore can't be used to identify players.
    pub client_guid: i64,
}

impl UnconnectedPing {
    /// Unique identifier of this packet.
    pub const ID: u8 = 0x01;
}

impl Decodable for UnconnectedPing {
    fn decode(mut buffer: BytesMut) -> VexResult<Self> {
        vex_assert!(buffer.get_u8() == Self::ID);

        let time = buffer.get_i64();
        buffer.get_u128(); // Skip offline message data
        let client_guid = buffer.get_i64();

        Ok(Self { time, client_guid })
    }
}
