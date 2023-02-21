use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use bytes::{BufMut, Bytes, BytesMut};

use crate::instance_manager::IPV4_LOCAL_ADDR;
use common::Serialize;
use common::VResult;
use common::{
    WriteExtensions, EMPTY_IPV4_ADDRESS, IPV4_MEM_SIZE, IPV6_MEM_SIZE,
};

/// Sent in response to [`ConnectionRequest`](super::connection_request::ConnectionRequest).
#[derive(Debug)]
pub struct ConnectionRequestAccepted {
    /// IP address of the client.
    pub client_address: SocketAddr,
    /// Corresponds to [`ConnectionRequest::time`](super::connection_request::ConnectionRequest::time).
    pub request_time: i64,
}

impl ConnectionRequestAccepted {
    /// Unique ID of this packet.
    pub const ID: u8 = 0x10;

    pub fn serialized_size(&self) -> usize {
        1 + IPV6_MEM_SIZE + 2 + 10 * IPV4_MEM_SIZE + 8 + 8
    }
}

impl Serialize for ConnectionRequestAccepted {
    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_u8(Self::ID);
        buffer.put_addr(self.client_address);
        buffer.put_i16(0); // System index
        for _ in 0..20 {
            // 20 internal IDs
            buffer.put_addr(*EMPTY_IPV4_ADDRESS);
        }
        buffer.put_i64(self.request_time);
        buffer.put_i64(self.request_time); // Response time
    }
}
