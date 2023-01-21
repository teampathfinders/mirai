use bytes::{Buf, BytesMut};

use crate::error::VexResult;
use crate::raknet::packets::Decodable;
use crate::vex_assert;

#[derive(Debug)]
pub struct ConnectionRequest {
    pub guid: i64,
    pub time: i64,
}

impl ConnectionRequest {
    pub const ID: u8 = 0x09;
}

impl Decodable for ConnectionRequest {
    fn decode(mut buffer: BytesMut) -> VexResult<Self> {
        vex_assert!(buffer.get_u8() == Self::ID);

        let guid = buffer.get_i64();
        let time = buffer.get_i64();

        Ok(Self { guid, time })
    }
}
