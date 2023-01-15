use std::net::SocketAddr;
use bytes::{Buf, BytesMut};
use crate::error::VexResult;
use crate::raknet::packets::Decodable;
use crate::util::ReadAddress;
use crate::vex_assert;

#[derive(Debug)]
pub struct OpenConnectionRequest2 {
    pub mtu: u16,
    pub client_guid: i64,
}

impl OpenConnectionRequest2 {
    pub const ID: u8 = 0x07;
}

impl Decodable for OpenConnectionRequest2 {
    fn decode(mut buffer: BytesMut) -> VexResult<Self> {
        vex_assert!(buffer.get_u8() == Self::ID);

        buffer.get_u128(); // Skip magic
        buffer.get_addr()?; // Skip server address
        let mtu = buffer.get_u16();
        let client_guid = buffer.get_i64();

        Ok(Self {
            mtu, client_guid
        })
    }
}