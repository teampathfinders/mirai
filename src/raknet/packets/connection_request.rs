#[derive(Debug)]
pub struct ConnectionRequest {
    pub guid: i64,
    pub time: i64,
}

impl ConnectionRequest {
    pub const ID: u8 = 0x09;
}
