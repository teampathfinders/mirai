use util::{BinaryRead};
use util::iassert;
use util::Deserialize;


/// Sent by the client to initiate a full connection.
/// [`ConnectionRequestAccepted`](crate::raknet::ConnectionRequestAccepted) should be sent in response.
#[derive(Debug)]
pub struct ConnectionRequest {
    /// Client-provided GUID.
    pub guid: i64,
    /// Timestamp of when this packet was sent.
    pub time: i64,
}

impl ConnectionRequest {
    /// Unique ID of this packet.
    pub const ID: u8 = 0x09;
}

impl<'a> Deserialize<'a> for ConnectionRequest {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        iassert!(reader.read_u8()? == Self::ID);

        let guid = reader.read_i64_be()?;
        let time = reader.read_i64_be()?;

        Ok(Self { guid, time })
    }
}
