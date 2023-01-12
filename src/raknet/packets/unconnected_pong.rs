use crate::raknet::packets::{Encodable, RaknetPacket, OFFLINE_MESSAGE_DATA};
use crate::{__impl_general_packet, encodable};
use bytes::{BufMut, BytesMut};

encodable!(
    0x1c,
    pub struct UnconnectedPong<'a> {
        time: i64,
        server_guid: i64,
        metadata: &'a str
    }
);

impl Encodable for UnconnectedPong<'_> {
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
