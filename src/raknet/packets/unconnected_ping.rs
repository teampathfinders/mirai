use crate::error::VexResult;
use crate::raknet::packets::{Decodable, Encodable};
use crate::{vex_assert};
use bytes::{Buf, BufMut, BytesMut};

#[derive(Debug)]
pub struct UnconnectedPing {
    pub time: i64,
    pub client_guid: i64,
}

impl UnconnectedPing {
    pub const ID: u8 = 0x01;
}

impl Decodable for UnconnectedPing {
    fn decode(mut buffer: BytesMut) -> VexResult<Self> {
        vex_assert!(buffer.get_u8() == Self::ID);

        let time = buffer.get_i64();
        buffer.get_u128(); // Skip offline message data
        let client_guid = buffer.get_i64();

        Ok(Self { time, client_guid })
    }
}
