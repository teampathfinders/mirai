use bytes::{Buf, BytesMut};

use crate::error::VexResult;
use crate::network::traits::Decodable;
use crate::vex_assert;

#[derive(Debug)]
pub struct OnlinePing {
    pub time: i64,
}

impl OnlinePing {
    pub const ID: u8 = 0x00;
}

impl Decodable for OnlinePing {
    fn decode(mut buffer: BytesMut) -> VexResult<Self> {
        vex_assert!(buffer.get_u8() == Self::ID);

        let time = buffer.get_i64();

        Ok(Self { time })
    }
}