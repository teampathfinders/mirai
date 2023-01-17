use crate::error::VexResult;
use crate::raknet::packets::Decodable;
use crate::vex_assert;
use bytes::{Buf, BytesMut};

#[derive(Debug)]
pub struct OpenConnectionRequest1 {
    pub protocol_version: u8,
    pub mtu: u16,
}

impl OpenConnectionRequest1 {
    pub const ID: u8 = 0x05;
}

impl Decodable for OpenConnectionRequest1 {
    fn decode(mut buffer: BytesMut) -> VexResult<Self> {
        vex_assert!(buffer.get_u8() == Self::ID);

        buffer.get_u128(); // Skip magic
        let protocol_version = buffer.get_u8();
        let mtu = buffer.len() as u16 - 18 + 46; // Size of padding + 46

        Ok(Self {
            protocol_version,
            mtu,
        })
    }
}
