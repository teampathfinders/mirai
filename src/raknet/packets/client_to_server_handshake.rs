#[derive(Debug)]
pub struct ClientToServerHandshake {
    pub jwt_data: i32,
    // This packet has no data.
}

impl ClientToServerHandshake {
    pub const ID: u8 = 0x04;
}
