use std::net::SocketAddr;

use bytes::{Buf, BytesMut};

use common::nvassert;
use common::Deserialize;
use common::VResult;
use common::{ReadExtensions, EMPTY_IPV4_ADDRESS};

/// Confirms that the connection was successfully initiated.
#[derive(Debug)]
pub struct NewIncomingConnection;

impl NewIncomingConnection {
    /// Unique ID of this packet.
    pub const ID: u8 = 0x13;
}

impl Deserialize for NewIncomingConnection {
    fn deserialize(mut buffer: BytesMut) -> VResult<Self> {
        nvassert!(buffer.get_u8() == Self::ID);

        // No data in this packet is used, there is no point in decoding it
        Ok(Self)
    }
}
