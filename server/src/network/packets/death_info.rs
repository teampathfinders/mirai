use bytes::BytesMut;
use common::{VResult, WriteExtensions};

use crate::network::Encodable;

use super::GamePacket;

#[derive(Debug)]
pub struct DeathInfo {
    pub cause: String,
    pub messages: Vec<String>,
}

impl GamePacket for DeathInfo {
    const ID: u32 = 0xbd;
}

impl Encodable for DeathInfo {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_string(&self.cause);

        buffer.put_var_u32(self.messages.len() as u32);
        for message in &self.messages {
            buffer.put_string(message);
        }

        Ok(buffer)
    }
}
