use std::net::SocketAddr;

use bytes::Bytes;
use bytes::{Buf, BytesMut};

use util::nvassert;
use util::Deserialize;
use util::Result;

/// Confirms that the connection was successfully initiated.
#[derive(Debug)]
pub struct NewIncomingConnection;

impl NewIncomingConnection {
    /// Unique ID of this packet.
    pub const ID: u8 = 0x13;
}

impl Deserialize for NewIncomingConnection {
    fn deserialize(mut buffer: Bytes) -> Result<Self> {
        nvassert!(buffer.get_u8() == Self::ID);

        // No data in this packet is used, there is no point in decoding it
        Ok(Self)
    }
}
