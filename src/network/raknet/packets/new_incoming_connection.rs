use std::net::SocketAddr;

use bytes::{Buf, BytesMut};

use crate::error::VexResult;
use crate::network::traits::Decodable;
use crate::util::{ReadExtensions, EMPTY_IPV4_ADDRESS};
use crate::vex_assert;

#[derive(Debug)]
pub struct NewIncomingConnection {
    pub server_address: SocketAddr,
    pub client_timestamp: i64,
    pub server_timestamp: i64,
}

impl NewIncomingConnection {
    pub const ID: u8 = 0x13;
}

impl Decodable for NewIncomingConnection {
    fn decode(mut buffer: BytesMut) -> VexResult<Self> {
        vex_assert!(buffer.get_u8() == Self::ID);

        let server_address = buffer.get_addr()?;
        for _ in 0..20 {
            buffer.get_addr()?;
        }

        let client_timestamp = buffer.get_i64();
        let server_timestamp = buffer.get_i64();

        Ok(Self {
            server_address,
            client_timestamp,
            server_timestamp,
        })
    }
}
