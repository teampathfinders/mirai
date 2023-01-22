/// Sent by the client to disconnect from the server.
#[derive(Debug)]
pub struct DisconnectNotification;

impl DisconnectNotification {
    /// Unique ID of this packet.
    pub const ID: u8 = 0x15;
}
