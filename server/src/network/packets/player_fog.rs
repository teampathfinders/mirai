use bytes::BytesMut;
use common::{Encodable, VResult, WriteExtensions};

use super::GamePacket;

#[derive(Debug)]
pub struct PlayerFog {
    /// Lists of fog identifiers
    pub stack: Vec<String>
}

impl GamePacket for PlayerFog {
    const ID: u32 = 0xa0;
}

impl Encodable for PlayerFog {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_var_u32(self.stack.len() as u32);
        for fog in &self.stack {
            buffer.put_string(fog);
        }

        Ok(buffer)
    }
}