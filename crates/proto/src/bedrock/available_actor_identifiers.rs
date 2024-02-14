use crate::bedrock::ConnectedPacket;

/// Lets the client know about the entities available on the server.
#[derive(Debug, Clone)]
pub struct AvailableActorIdentifiers<'a> {
    /// Serialised NBT structure containing the entities.
    pub identifiers: &'a [u8],
}

impl<'a> ConnectedPacket for AvailableActorIdentifiers<'a> {
    const ID: u32 = 0x77;
}