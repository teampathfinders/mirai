#[derive(Debug)]
pub struct RaknetDisconnect;

impl RaknetDisconnect {
    pub const ID: u8 = 0x15;
}

#[derive(Debug)]
pub struct Disconnect {
    pub hide_disconnect_screen: bool,
    pub kick_message: String, // This packet has no data.
}

impl Disconnect {
    pub const ID: u8 = 0x05;
}
