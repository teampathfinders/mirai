use std::net::SocketAddr;

use bytes::{Buf, BytesMut};

use crate::error::VexResult;
use crate::raknet::packets::Decodable;
use crate::util::ReadExtensions;
use crate::vex_assert;

#[derive(Debug)]
pub struct NewIncomingConnection {
    pub server_address: SocketAddr,
    pub internal_address: SocketAddr,
}

impl NewIncomingConnection {
    pub const ID: u8 = 0x13;
}

impl Decodable for NewIncomingConnection {
    fn decode(mut buffer: BytesMut) -> VexResult<Self> {
        vex_assert!(buffer.get_u8() == Self::ID);

        let server_address = buffer.get_addr()?;
        let internal_address = buffer.get_addr()?;

        Ok(Self {
            server_address,
            internal_address,
        })
    }
}