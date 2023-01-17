#[derive(Debug)]
pub struct Login {
    pub protocol_version: i32,
    pub chain_data: i32,
    // JSON array of JWT Data,
    pub skin_data: u16,
    //     	JWT Data
}

impl Login {
    pub const ID: u8 = 0x01;
}
