use util::bytes::{BinaryRead, SharedBuffer};
use util::pyassert;
use util::Deserialize;
use util::Result;

/// Sent to retrieve information about the server
/// while the user is in Minecraft's server tab.
/// An [`OfflinePong`](crate::offline_pong::OfflinePong) packet should be sent in response.
#[derive(Debug)]
pub struct UnconnectedPing {
    /// Time when this ping was sent.
    /// Used to measure server latency.
    pub time: u64,
    /// GUID of the client.
    ///
    /// Unknown what this is used for.
    /// It's randomised each time Minecraft is restarted and therefore can't be used to identify players.
    pub client_guid: u64,
}

impl UnconnectedPing {
    /// Unique identifier of this packet.
    pub const ID: u8 = 0x01;
}

impl Deserialize<'_> for UnconnectedPing {
    fn deserialize(mut buffer: SharedBuffer) -> anyhow::Result<Self> {
        pyassert!(buffer.read_u8()? == Self::ID);

        let time = buffer.read_u64_be()?;
        buffer.advance(16); // Skip offline message data
        let client_guid = buffer.read_u64_be()?;

        Ok(Self { time, client_guid })
    }
}
