use bytes::Bytes;
use bytes::{BufMut, BytesMut};

use crate::network::raknet::OFFLINE_MESSAGE_DATA;
use common::Serialize;
use common::VResult;

/// Sent in response to [`OpenConnectionRequest1`](super::open_connection_request1::OpenConnectionRequest1).
#[derive(Debug)]
pub struct OpenConnectionReply1 {
    /// GUID of the server.
    /// Corresponds to [`ServerInstance::guid`](crate::ServerInstance::guid).
    pub server_guid: i64,
    /// MTU of the connection.
    /// This should be given the same value as [`OpenConnectionRequest1::mtu`](super::open_connection_request1::OpenConnectionRequest1::mtu).
    pub mtu: u16,
}

impl OpenConnectionReply1 {
    /// Unique identifier of this packet.
    pub const ID: u8 = 0x06;
}

impl Serialize for OpenConnectionReply1 {
    fn serialize(&self) -> VResult<Bytes> {
        let mut buffer = BytesMut::with_capacity(1 + 16 + 8 + 1 + 2);

        buffer.put_u8(Self::ID);
        buffer.put(OFFLINE_MESSAGE_DATA);
        buffer.put_i64(self.server_guid);
        // Disable security, required for login sequence.
        // Encryption will be enabled later on.
        buffer.put_u8(0);
        buffer.put_u16(self.mtu);

        Ok(buffer.freeze())
    }
}
