/// Sent by the server to disconnect a client.
#[derive(Debug)]
pub struct Disconnect {
    /// Whether to immediately send the client to the main menu.
    pub hide_disconnect_screen: bool,
    pub kick_message: String, // This packet has no data.
}

impl Disconnect {
    pub const ID: u8 = 0x05;
}
