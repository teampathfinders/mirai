
#[derive(Debug)]
pub struct OpenConnectionRequest1 {
    pub protocol_version: u8,
    pub mtu: u16,
}

impl OpenConnectionRequest1 {
    pub const ID: u8 = 0x05;
}
