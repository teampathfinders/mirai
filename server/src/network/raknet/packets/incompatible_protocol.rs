use bytes::{BufMut, BytesMut};

use crate::network::raknet::{OFFLINE_MESSAGE_DATA, RAKNET_VERSION};
use crate::network::traits::Encodable;
use common::VResult;

/// Notifies the client that they're using a version of the Raknet protocol that is incompatible
/// with the version used by the server ([`RAKNET_VERSION`]).
///
/// This packet should be sent in response to [`OpenConnectionRequest1`](super::open_connection_request1::OpenConnectionRequest1)
/// if the [`protocol_version`](super::open_connection_request1::OpenConnectionRequest1::protocol_version) field does not match the server's version.
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
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(1 + 1 + 16 + 8);

        buffer.put_u8(Self::ID);
        buffer.put_u8(RAKNET_VERSION);
        buffer.put(OFFLINE_MESSAGE_DATA);
        buffer.put_i64(self.server_guid);

        Ok(buffer)
    }
}
