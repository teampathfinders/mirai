use bytes::BytesMut;
use common::{VResult, WriteExtensions};

use crate::network::Encodable;

use super::GamePacket;

/// Information about a player's death.
#[derive(Debug)]
pub struct DeathInfo {
    /// Cause of death.
    pub cause: String,
    /// Additional info display in the death screen.
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
