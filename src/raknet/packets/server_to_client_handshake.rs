#[derive(Debug)]
pub struct ServerToClientHandshake {
    pub jwt_data: i32,
    // JWT String
}

impl ServerToClientHandshake {
    pub const ID: u8 = 0x03;
}
