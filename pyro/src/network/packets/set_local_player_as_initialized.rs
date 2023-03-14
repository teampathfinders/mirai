use bytes::{BytesMut, Bytes};
use util::{Result};
use util::bytes::ReadBuffer;

use util::Deserialize;

use super::ConnectedPacket;

/// Sent by the client to indicate that the player has been fully initialised.
#[derive(Debug, Clone)]
pub struct SetLocalPlayerAsInitialized {
    /// Runtime ID of the player.
    pub runtime_id: u64,
}

impl ConnectedPacket for SetLocalPlayerAsInitialized {
    const ID: u32 = 0x71;
}

impl Deserialize for SetLocalPlayerAsInitialized {
    fn deserialize(mut buffer: ReadBuffer) -> Result<Self> {
        Ok(Self { runtime_id: buffer.read_var::<u64>()? })
    }
}
