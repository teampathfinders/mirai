
use util::Serialize;
use crate::network::packets::ConnectedPacket;

/// Lets the client know about the entities available on the server.
#[derive(Debug, Clone)]
pub struct AvailableActorIdentifiers {
    /// Serialised NBT structure containing the entities.
    pub identifiers: SharedBuffer
}

impl ConnectedPacket for AvailableActorIdentifiers {
    const ID: u32 = 0x77;
}