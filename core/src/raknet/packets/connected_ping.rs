use util::bytes::{BinaryRead, SharedBuffer};
use util::Deserialize;
use util::pyassert;
use util::Result;

/// Sent by the client or server to ping the other side.
/// An [`OnlinePong`](crate::OnlinePong) packet should be sent in response.
#[derive(Debug)]
pub struct ConnectedPing {
    /// Timestamp of when the ping was sent.
    pub time: i64,
}

impl ConnectedPing {
    /// Unique ID of this packet.
    pub const ID: u8 = 0x00;
}

impl<'a> Deserialize<'a> for ConnectedPing {
    fn deserialize<R>(mut reader: R) -> anyhow::Result<Self> where R: BinaryRead<'a> + 'a {
        pyassert!(reader.read_u8()? == Self::ID);

        let time = reader.read_i64_be()?;

        Ok(Self { time })
    }
}
