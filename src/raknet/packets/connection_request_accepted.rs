use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use bytes::{BufMut, BytesMut};

use crate::error::VexResult;
use crate::instance::IPV4_LOCAL_ADDR;
use crate::raknet::packets::Encodable;
use crate::util::{EMPTY_IPV4_ADDRESS, IPV4_MEM_SIZE, IPV6_MEM_SIZE, WriteExtensions};

#[derive(Debug)]
pub struct ConnectionRequestAccepted {
    pub client_address: SocketAddr,
    pub request_time: i64,
}

impl ConnectionRequestAccepted {
    pub const ID: u8 = 0x10;
}

impl Encodable for ConnectionRequestAccepted {
    fn encode(&self) -> VexResult<BytesMut> {
        let mut buffer =
            BytesMut::with_capacity(1 + IPV6_MEM_SIZE + 2 + 10 * IPV4_MEM_SIZE + 8 + 8);

        buffer.put_u8(Self::ID);
        buffer.put_addr(self.client_address);
        buffer.put_i16(0); // System index
        for _ in 0..10 {
            // 10 internal IDs
            buffer.put_addr(*EMPTY_IPV4_ADDRESS);
        }
        buffer.put_i64(self.request_time);
        buffer.put_i64(self.request_time); // Response time

        Ok(buffer)
    }
}
