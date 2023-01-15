
use std::net::SocketAddr;
use bytes::{BufMut, BytesMut};
use crate::error::VexResult;
use crate::raknet::packets::{Encodable, OFFLINE_MESSAGE_DATA};
use crate::util::WriteAddress;

#[derive(Debug)]
pub struct OpenConnectionReply2 {
    pub server_guid: i64,
    pub client_address: SocketAddr,
    pub mtu: u16,
    pub encryption_enabled: bool,
}

impl OpenConnectionReply2 {
    pub const ID: u8 = 0x08;
}

impl Encodable for OpenConnectionReply2 {
    fn encode(&self) -> VexResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(1 + 16 + 8 + 1 + 16 + 2 + 2 + 1);

        buffer.put_u8(Self::ID);
        buffer.put(OFFLINE_MESSAGE_DATA);
        buffer.put_i64(self.server_guid);
        buffer.put_addr(self.client_address);
        buffer.put_u16(self.mtu);
        buffer.put_u8(self.encryption_enabled as u8);

        Ok(buffer)
    }
}