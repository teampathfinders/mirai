use bytes::BytesMut;
use common::{VResult, WriteExtensions, size_of_var};

use common::Encodable;

use super::GamePacket;

/// Information about a player's death.
#[derive(Debug, Clone)]
pub struct DeathInfo<'a> {
    /// Cause of death.
    pub cause: &'a str,
    /// Additional info display in the death screen.
    pub messages: &'a [String],
}

impl GamePacket for DeathInfo<'_> {
    const ID: u32 = 0xbd;
}

impl Encodable for DeathInfo<'_> {
    fn encode(&self) -> VResult<BytesMut> {
        let packet_size = size_of_var(self.cause.len() as u32) + self.cause.len() +
        size_of_var(self.messages.len() as u32) + 
        self.messages.iter().fold(0, |acc, m| acc + size_of_var(m.len() as u32) + m.len());

        let mut buffer = BytesMut::with_capacity(
            packet_size    
        );

        buffer.put_string(self.cause);

        buffer.put_var_u32(self.messages.len() as u32);
        for message in self.messages {
            buffer.put_string(message);
        }

        Ok(buffer)
    }
}
