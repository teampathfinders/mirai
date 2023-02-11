use bytes::{BufMut, BytesMut};

use common::VResult;
use crate::network::traits::Encodable;

/// Sent by the server or client in response to an [`OnlinePing`](super::OnlinePing) packet.
#[derive(Debug)]
pub struct OnlinePong {
    /// Timestamp of when the ping was sent.
    pub ping_time: i64,
    /// Current time.
    pub pong_time: i64,
}

impl OnlinePong {
    /// Unique ID of this packet.
    pub const ID: u8 = 0x03;
}

impl Encodable for OnlinePong {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(1 + 8 + 8);

        buffer.put_u8(Self::ID);
        buffer.put_i64(self.ping_time);
        buffer.put_i64(self.pong_time);

        Ok(buffer)
    }
}
