use crate::error::VexResult;
use crate::raknet::packets::{Encodable, OFFLINE_MESSAGE_DATA};
use bytes::{BufMut, BytesMut};

#[derive(Debug)]
pub struct OpenConnectionReply1 {
    pub server_guid: i64,
    pub mtu: u16,
}

impl OpenConnectionReply1 {
    pub const ID: u8 = 0x06;
}

impl Encodable for OpenConnectionReply1 {
    fn encode(&self) -> VexResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(1 + 16 + 8 + 1 + 2);

        buffer.put_u8(Self::ID);
        buffer.put(OFFLINE_MESSAGE_DATA);
        buffer.put_i64(self.server_guid);
        buffer.put_u8(0); // Disable security, required for login sequence
        buffer.put_u16(self.mtu);

        Ok(buffer)
    }
}
