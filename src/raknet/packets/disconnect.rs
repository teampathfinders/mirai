#[derive(Debug)]
pub struct Disconnect {
    pub hide_disconnect_screen: bool,
    pub kick_message: String, // This packet has no data.
}

impl Disconnect {
    pub const ID: u8 = 0x05;
}
