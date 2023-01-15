use crate::error::VexResult;
use crate::raknet::packets::{Decodable, Encodable};
use crate::{vex_check};
use bytes::{Buf, BufMut, BytesMut};

decodable!(
    0x01,
    pub struct UnconnectedPing {
        time: i64,
        client_guid: i64,
    }
);

impl Decodable for UnconnectedPing {
    fn decode(mut buffer: BytesMut) -> VexResult<Self> {
        vex_check!(buffer.get_u8() == Self::ID);

        let time = buffer.get_i64();
        buffer.get_u128(); // Skip offline message data
        let client_guid = buffer.get_i64();

        Ok(Self { time, client_guid })
    }
}
