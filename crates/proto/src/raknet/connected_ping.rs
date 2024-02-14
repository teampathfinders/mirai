use util::{BinaryRead};
use util::Deserialize;
use util::iassert;


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

impl<'a> Deserialize<'a> for ConnectedPing {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        iassert!(reader.read_u8()? == Self::ID);

        let time = reader.read_i64_be()?;

        Ok(Self { time })
    }
}
