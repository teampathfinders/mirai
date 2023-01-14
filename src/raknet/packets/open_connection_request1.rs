use crate::decodable;


decodable!(
     0x05,
    pub struct OpenConnectionRequest1{
        protocol_version: u8,
        mtu: u16
    }
);

