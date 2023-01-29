use std::net::SocketAddr;

use bytes::{Buf, BytesMut};

use crate::network::traits::Decodable;
use crate::util::{EMPTY_IPV4_ADDRESS, ReadExtensions};
use crate::vex_assert;

/// Confirms that the connection was successfully initiated.
#[derive(Debug)]
pub struct NewIncomingConnection;

impl NewIncomingConnection {
    /// Unique ID of this packet.
    pub const ID: u8 = 0x13;
}

impl Decodable for NewIncomingConnection {
    fn decode(mut buffer: BytesMut) -> anyhow::Result<Self> {
        vex_assert!(buffer.get_u8() == Self::ID);

        // No data in this packet is used, there is no point in decoding it
        Ok(Self)
    }
}
