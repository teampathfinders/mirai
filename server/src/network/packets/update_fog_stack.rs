use bytes::BytesMut;
use common::{Encodable, VResult, WriteExtensions};

use super::GamePacket;

/// Adds a fog to the client's fog stack.
#[derive(Debug, Clone)]
pub struct UpdateFogStack<'s> {
    /// Lists of fog identifiers
    pub stack: &'s [String],
}

impl GamePacket for UpdateFogStack<'_> {
    const ID: u32 = 0xa0;
}

impl Encodable for UpdateFogStack<'_> {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_var_u32(self.stack.len() as u32);
        for fog in self.stack {
            buffer.put_string(fog);
        }

        Ok(buffer)
    }
}
