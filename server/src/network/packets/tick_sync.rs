use bytes::{Buf, BufMut, BytesMut};
use common::VResult;

use crate::network::{Decodable, Encodable};

use super::GamePacket;

/// Synchronises the current tick.
/// 
/// This packet is first sent by the client and should be responded to with the same request timestamp and a new response timestamp.
#[derive(Debug)]
pub struct TickSync {
    /// Timestamp of when the client sent the packet.
    pub request: u64,
    /// Timestamp of when the server sent the packet.
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
