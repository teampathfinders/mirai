use bytes::Bytes;
use bytes::{BufMut, BytesMut};

use common::Serialize;
use common::VResult;

/// Sent by the server or client in response to an [`OnlinePing`](super::OnlinePing) packet.
#[derive(Debug)]
pub struct ConnectedPong {
    /// Timestamp of when the ping was sent.
    pub ping_time: i64,
    /// Current time.
    pub pong_time: i64,
}

impl ConnectedPong {
    /// Unique ID of this packet.
    pub const ID: u8 = 0x03;

    pub fn serialized_size(&self) -> usize {
        1 + 8 + 8
    }
}

impl Serialize for ConnectedPong {
    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_u8(Self::ID);
        buffer.put_i64(self.ping_time);
        buffer.put_i64(self.pong_time);
    }
}
