use util::BinaryRead;
use util::Deserialize;


use crate::bedrock::ConnectedPacket;

/// Sent by the client in response to a [`ServerToClientHandshake`](crate::bedrock::ServerToClientHandshake)
/// to confirm that encryption is working.
///
/// It has no data.
#[derive(Debug)]
pub struct ClientToServerHandshake;

impl ConnectedPacket for ClientToServerHandshake {
    /// Unique ID of this packet.
    const ID: u32 = 0x04;
}

impl<'a> Deserialize<'a> for ClientToServerHandshake {
    fn deserialize_from<R: BinaryRead<'a>>(_reader: &mut R) -> anyhow::Result<Self> {
        Ok(Self)
    }
}
