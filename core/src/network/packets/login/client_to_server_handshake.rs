use util::bytes::BinaryRead;
use util::bytes::SharedBuffer;
use util::Deserialize;
use util::Result;

use crate::network::ConnectedPacket;

/// Sent by the client in response to a [`ServerToClientHandshake`](crate::ServerToClientHandshake)
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
    fn deserialize<R>(_reader: R) -> anyhow::Result<Self>
    where
        R: BinaryRead<'a> + 'a
    {
        Ok(Self)
    }
}
