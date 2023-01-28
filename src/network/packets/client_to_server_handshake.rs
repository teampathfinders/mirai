/// Sent by the client in response to a [`ServerToClientHandshake`](super::ServerToClientHandshake)
/// to confirm that encryption is working.
///
/// It has no data.
#[derive(Debug)]
pub struct ClientToServerHandshake;

impl ClientToServerHandshake {
    /// Unique ID of this packet.
    pub const ID: u8 = 0x04;
}
