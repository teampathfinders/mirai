use std::net::SocketAddr;

use bytes::{Buf, BytesMut};

use common::vassert;
use common::Decodable;
use common::VResult;
use common::{ReadExtensions, EMPTY_IPV4_ADDRESS};

/// Confirms that the connection was successfully initiated.
#[derive(Debug, Clone)]
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
