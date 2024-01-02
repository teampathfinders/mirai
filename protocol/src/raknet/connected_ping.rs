use util::{BinaryRead, SharedBuffer};
use util::Deserialize;
use util::pyassert;
use util::Result;

/// Sent by the client or server to ping the other side.
/// An [`ConnectedPong`](crate::raknet::ConnectedPong) packet should be sent in response.
#[derive(Debug)]
pub struct ConnectedPing {
    /// Timestamp of when the ping was sent.
    pub time: i64,
}

impl ConnectedPing {
    /// Unique ID of this packet.
    pub const ID: u8 = 0x00;
}

impl Deserialize<'_> for ConnectedPing {
    fn deserialize(mut buffer: SharedBuffer) -> anyhow::Result<Self> {
        pyassert!(buffer.read_u8()? == Self::ID);

        let time = buffer.read_i64_be()?;

        Ok(Self { time })
    }
}
