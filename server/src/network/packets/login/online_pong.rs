use bytes::{BufMut, BytesMut};

use common::Serialize;
use common::VResult;

/// Sent by the server or client in response to an [`OnlinePing`](super::OnlinePing) packet.
#[derive(Debug, Clone)]
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

impl Serialize for OnlinePong {
    fn serialize(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(1 + 8 + 8);

        buffer.put_u8(Self::ID);
        buffer.put_i64(self.ping_time);
        buffer.put_i64(self.pong_time);

        Ok(buffer)
    }
}
