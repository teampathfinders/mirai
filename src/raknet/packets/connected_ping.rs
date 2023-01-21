use bytes::{Buf, BytesMut};

use crate::error::VexResult;
use crate::raknet::packets::Decodable;
use crate::vex_assert;

#[derive(Debug)]
pub struct ConnectedPing {
    pub time: i64,
}

impl ConnectedPing {
    pub const ID: u8 = 0x00;
}

impl Decodable for ConnectedPing {
    fn decode(mut buffer: BytesMut) -> VexResult<Self> {
        vex_assert!(buffer.get_u8() == Self::ID);

        let time = buffer.get_i64();

        Ok(Self {
            time
        })
    }
}