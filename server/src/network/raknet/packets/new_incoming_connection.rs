use std::net::SocketAddr;

use bytes::{Buf, BytesMut};

use common::VResult;
use crate::network::traits::Decodable;
use common::{EMPTY_IPV4_ADDRESS, ReadExtensions};
use common::vassert;

/// Confirms that the connection was successfully initiated.
#[derive(Debug)]
pub struct NewIncomingConnection;

impl NewIncomingConnection {
    /// Unique ID of this packet.
    pub const ID: u8 = 0x13;
}

impl Decodable for NewIncomingConnection {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        vassert!(buffer.get_u8() == Self::ID);

        // No data in this packet is used, there is no point in decoding it
        Ok(Self)
    }
}
