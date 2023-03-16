use util::bytes::SharedBuffer;

use crate::ConnectedPacket;

/// Lets the client know about the entities available on the server.
#[derive(Debug, Clone)]
pub struct AvailableActorIdentifiers<'a> {
    /// Serialised NBT structure containing the entities.
    pub identifiers: SharedBuffer<'a>
}

impl<'a> ConnectedPacket for AvailableActorIdentifiers<'a> {
    const ID: u32 = 0x77;
}