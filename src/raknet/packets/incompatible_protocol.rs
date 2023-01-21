use bytes::{BufMut, BytesMut};

use crate::error::VexResult;
use crate::raknet::packets::{Encodable, OFFLINE_MESSAGE_DATA, RAKNET_VERSION};

/// Notifies the client that they're using a version of the Raknet protocol that is incompatible
/// with the version used by the server ([`RAKNET_VERSION`]).
///
/// This packet should be sent in response to [`OpenConnectionRequest1`](super::OpenConnectionRequest1)
/// if the protocol_version field does not match the server's version.
pub struct IncompatibleProtocol {
    /// Randomly generated GUID of the server.
    /// Corresponds to [`ServerInstance::guid`](crate::ServerInstance::guid).
    pub server_guid: i64,
}

impl IncompatibleProtocol {
    /// Unique idrentifier of this packet.
    pub const ID: u8 = 0x19;
}

impl Encodable for IncompatibleProtocol {
    fn encode(&self) -> VexResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(1 + 1 + 16 + 8);

        buffer.put_u8(Self::ID);
        buffer.put_u8(RAKNET_VERSION);
        buffer.put(OFFLINE_MESSAGE_DATA);
        buffer.put_i64(self.server_guid);

        Ok(buffer)
    }
}
