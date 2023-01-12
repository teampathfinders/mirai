use bytes::BytesMut;
use crate::encodable;
use crate::packets::{Encodable, UnconnectedPing};

encodable!(
    0x1c,
    pub struct UnconnectedPong<'a> {
        time: u64,
        server: u64,
        server_id: &'a str
    }
);

impl Encodable for UnconnectedPong<'_> {
    fn encode(&self) -> BytesMut {
        todo!()
    }
}

