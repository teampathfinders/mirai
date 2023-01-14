use crate::decodable;

pub struct OpenConnectionRequest1 {
    protocol_version: u8,
    mtu: u16,
}

impl OpenConnectionRequest1 {
    const ID: u8 = 0x05;
}
