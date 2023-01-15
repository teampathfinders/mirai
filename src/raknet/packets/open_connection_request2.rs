
use std::net::SocketAddr;
use bytes::BytesMut;
use crate::error::VexResult;
use crate::raknet::packets::Decodable;
use crate::vex_assert;

pub struct OpenConnectionRequest2 {
    pub server_address: SocketAddr,
    pub mtu: u16,
    pub client_guid: i64,
}

impl OpenConnectionRequest2 {
    pub const ID: u8 = 0x07;
}

impl Decodable for OpenConnectionRequest2 {
    fn decode(buffer: BytesMut) -> VexResult<Self> {
        vex_assert!(buffer.get_u8() == Self::ID);


    }
}