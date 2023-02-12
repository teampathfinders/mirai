use bytes::{Buf, BufMut, BytesMut};
use common::VResult;

use crate::network::{Decodable, Encodable};

use super::GamePacket;

#[derive(Debug)]
pub struct TickSync {
    pub request: u64,
    pub response: u64,
}

impl GamePacket for TickSync {
    const ID: u32 = 0x17;
}

impl Decodable for TickSync {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        let request = buffer.get_u64();
        let response = buffer.get_u64();

        Ok(Self { request, response })
    }
}

impl Encodable for TickSync {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(16);

        buffer.put_u64(self.request);
        buffer.put_u64(self.response);

        Ok(buffer)
    }
}
