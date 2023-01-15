use crate::raknet::packets::{Encodable, OFFLINE_MESSAGE_DATA};
use bytes::{BufMut, BytesMut};


    pub struct UnconnectedPong {
        time: i64,
        server_guid: i64,
        metadata: String,
    }
impl UnconnectedPong {
    pub const ID: u8 = 0x1c;
}

impl Encodable for UnconnectedPong {
    fn encode(&self) -> BytesMut {
        let mut buffer = BytesMut::with_capacity(1 + 8 + 8 + 16 + 2 + self.metadata.len());

        buffer.put_u8(Self::ID);
        buffer.put_i64(self.time);
        buffer.put_i64(self.server_guid);
        buffer.put(OFFLINE_MESSAGE_DATA);
        buffer.put_u16(self.metadata.len() as u16);
        buffer.put(self.metadata.as_bytes());

        buffer
    }
}
