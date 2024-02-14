use util::BinaryWrite;
use util::Serialize;

/// Sent by the server or client in response to an [`ConnectedPing`](crate::raknet::ConnectedPing) packet.
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

    /// Estimates the size of the packet when serialized.
    pub const fn size_hint(&self) -> usize {
        1 + 8 + 8
    }
}

impl Serialize for ConnectedPong {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_u8(Self::ID)?;
        writer.write_i64_be(self.ping_time)?;
        writer.write_i64_be(self.pong_time)
    }
}
