use bytes::Bytes;
use bytes::{Buf, BytesMut};

use common::nvassert;
use common::Deserialize;
use common::Result;

/// Sent by the client or server to ping the other side.
/// An [`OnlinePong`](super::OnlinePong) packet should be sent in response.
#[derive(Debug)]
pub struct ConnectedPing {
    /// Timestamp of when the ping was sent.
    pub time: i64,
}

impl ConnectedPing {
    /// Unique ID of this packet.
    pub const ID: u8 = 0x00;
}

impl Deserialize for ConnectedPing {
    fn deserialize(mut buffer: Bytes) -> Result<Self> {
        nvassert!(buffer.get_u8() == Self::ID);

        let time = buffer.get_i64();

        Ok(Self { time })
    }
}
