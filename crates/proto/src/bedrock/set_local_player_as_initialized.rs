use util::{BinaryRead};
use util::Deserialize;


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

impl<'a> Deserialize<'a> for SetLocalPlayerAsInitialized {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(Self { runtime_id: reader.read_var_u64()? })
    }
}
