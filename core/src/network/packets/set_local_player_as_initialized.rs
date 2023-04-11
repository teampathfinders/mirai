use util::bytes::{BinaryRead, SharedBuffer};
use util::Deserialize;
use util::Result;

use crate::network::ConnectedPacket;

/// Sent by the client to indicate that the player has been fully initialised.
#[derive(Debug, Clone)]
pub struct SetLocalPlayerAsInitialized {
    /// Runtime ID of the player.
    pub runtime_id: u64,
}

impl ConnectedPacket for SetLocalPlayerAsInitialized {
    const ID: u32 = 0x71;
}

impl<'a> Deserialize<'a> for SetLocalPlayerAsInitialized {
    fn deserialize<R>(reader: R) -> anyhow::Result<Self>
    where
        R: BinaryRead<'a> + 'a
    {
        Ok(Self { runtime_id: reader.read_var_u64()? })
    }
}
