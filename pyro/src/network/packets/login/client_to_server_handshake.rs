


use crate::network::packets::ConnectedPacket;
use util::nvassert;
use util::Deserialize;
use util::Result;

/// Sent by the client in response to a [`ServerToClientHandshake`](super::ServerToClientHandshake)
/// to confirm that encryption is working.
///
/// It has no data.
#[derive(Debug)]
pub struct ClientToServerHandshake;

impl ConnectedPacket for ClientToServerHandshake {
    /// Unique ID of this packet.
    const ID: u32 = 0x04;
}

impl Deserialize for ClientToServerHandshake {
    fn deserialize(mut buffer: SharedBuffer) -> Result<Self> {
        Ok(Self)
    }
}
