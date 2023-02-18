use bytes::Bytes;
use common::Serialize;
use crate::network::packets::GamePacket;

/// Lets the client know about the entities available on the server.
#[derive(Debug, Clone)]
pub struct AvailableActorIdentifiers {
    /// Serialised NBT structure containing the entities.
    pub identifiers: Bytes
}

impl GamePacket for AvailableActorIdentifiers {
    const ID: u32 = 0x77;
}