use bytes::{BytesMut, Bytes};
use common::{ReadExtensions, VResult};

use common::Deserialize;

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
    fn deserialize(mut buffer: Bytes) -> VResult<Self> {
        Ok(Self { runtime_id: buffer.get_var_u64()? })
    }
}
