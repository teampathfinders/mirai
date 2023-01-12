use bytes::{BufMut, BytesMut};
use crate::encodable;
use crate::packets::{Encodable, OFFLINE_MESSAGE_DATA, RaknetPacket, UnconnectedPing};

encodable!(
    0x1c,
    pub struct UnconnectedPong<'a> {
        time: i64,
        server_guid: i64,
        server_id: &'a str
    }
);

impl Encodable for UnconnectedPong<'_> {
    fn encode(&self) -> BytesMut {
        let mut buffer = BytesMut::with_capacity(
            1 + 8 + 8 + 16 + 2 + self.server_id.len()
        );

        buffer.put_u8(Self::ID);
        buffer.put_i64(self.time);
        buffer.put_i64(self.server_guid);
        buffer.put(OFFLINE_MESSAGE_DATA);
        buffer.put_u16(self.server_id.len() as u16);
        buffer.put(self.server_id.as_bytes());

        buffer
    }
}

