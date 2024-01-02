use util::{BinaryRead, SharedBuffer};
use util::Deserialize;
use util::Result;

use crate::bedrock::ConnectedPacket;

/// Sent by the client to indicate that the player has been fully initialised.
#[derive(Debug, Clone)]
pub struct SetLocalPlayerAsInitialized {
    /// Runtime ID of the player.
    pub runtime_id: u64,
}

impl ConnectedPacket for SetLocalPlayerAsInitialized {
    const ID: u32 = 0x71;
}

impl Deserialize<'_> for SetLocalPlayerAsInitialized {
    fn deserialize(mut buffer: SharedBuffer) -> anyhow::Result<Self> {
        Ok(Self { runtime_id: buffer.read_var_u64()? })
    }
}