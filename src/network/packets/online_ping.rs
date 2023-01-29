use bytes::{Buf, BytesMut};

use crate::network::traits::Decodable;
use crate::vex_assert;

/// Sent by the client or server to ping the other side.
/// An [`OnlinePong`](super::OnlinePong) packet should be sent in response.
#[derive(Debug)]
pub struct OnlinePing {
    /// Timestamp of when the ping was sent.
    pub time: i64,
}

impl OnlinePing {
    /// Unique ID of this packet.
    pub const ID: u8 = 0x00;
}

impl Decodable for OnlinePing {
    fn decode(mut buffer: BytesMut) -> anyhow::Result<Self> {
        vex_assert!(buffer.get_u8() == Self::ID);

        let time = buffer.get_i64();

        Ok(Self { time })
    }
}
