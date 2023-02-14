use bytes::BytesMut;
use common::{ReadExtensions, VResult};

use common::Decodable;

use super::GamePacket;

/// Sent by the client to indicate that the player has been fully initialised.
#[derive(Debug, Clone)]
pub struct SetLocalPlayerAsInitialized {
    /// Runtime ID of the player.
    pub runtime_id: u64,
}

impl GamePacket for SetLocalPlayerAsInitialized {
    const ID: u32 = 0x71;
}

impl Decodable for SetLocalPlayerAsInitialized {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        Ok(Self { runtime_id: buffer.get_var_u64()? })
    }
}
