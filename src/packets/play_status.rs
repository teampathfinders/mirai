#[derive(Debug)]
pub struct PlayStatus {
    pub status: i32,
}

impl PlayStatus {
    pub const ID: u8 = 0x02;
}
