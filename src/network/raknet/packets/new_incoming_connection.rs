use std::net::SocketAddr;

use bytes::{Buf, BytesMut};

use crate::error::VexResult;
use crate::network::traits::Decodable;
use crate::util::{EMPTY_IPV4_ADDRESS, ReadExtensions};
use crate::vex_assert;

/// Confirms that the connection was successfully initiated.
#[derive(Debug)]
pub struct NewIncomingConnection {
    /// IP address of the server.
    pub server_address: SocketAddr,
    pub client_timestamp: i64,
    pub server_timestamp: i64,
}

impl NewIncomingConnection {
    /// Unique ID of this packet.
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
