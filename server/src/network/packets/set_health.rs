use bytes::BytesMut;
use common::{VResult, WriteExtensions};

use crate::network::Encodable;

use super::GamePacket;

#[derive(Debug)]
pub struct SetHealth {
    pub health: i32,
}

impl GamePacket for SetHealth {
    const ID: u32 = 0x2a;
}

impl Encodable for SetHealth {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_var_i32(self.health);

        Ok(buffer)
    }
}
