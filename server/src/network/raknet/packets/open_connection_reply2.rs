use std::net::SocketAddr;

use bytes::Bytes;
use bytes::{BufMut, BytesMut};

use crate::network::raknet::OFFLINE_MESSAGE_DATA;
use common::Serialize;
use common::VResult;
use common::WriteExtensions;

/// Sent in response to [`OpenConnectionRequest2`](super::open_connection_request2::OpenConnectionRequest2).
#[derive(Debug)]
pub struct OpenConnectionReply2 {
    /// GUID of the server.
    /// Corresponds to [`ServerInstance::guid`](crate::ServerInstance::guid).
    pub server_guid: u64,
    /// IP address of the client.
    pub client_address: SocketAddr,
    /// MTU of the connection.
    /// This value should be the same as [`OpenConnectionRequest2::mtu`](super::open_connection_request2::OpenConnectionRequest2::mtu).
    pub mtu: u16,
}

impl OpenConnectionReply2 {
    /// Unique identifier of the packet.
    pub const ID: u8 = 0x08;
}

impl Serialize for OpenConnectionReply2 {
    fn serialize(&self) -> VResult<Bytes> {
        let mut buffer =
            BytesMut::with_capacity(1 + 16 + 8 + 1 + 16 + 2 + 2 + 1);

        buffer.put_u8(Self::ID);
        buffer.put(OFFLINE_MESSAGE_DATA);
        buffer.put_u64(self.server_guid);
        buffer.put_addr(self.client_address);
        buffer.put_u16(self.mtu);
        buffer.put_bool(false); // Encryption not enabled, must be false to continue login sequence

        Ok(buffer.freeze())
    }
}
