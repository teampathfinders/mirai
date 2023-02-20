use bytes::{BytesMut, Bytes};
use common::{Serialize, VResult, WriteExtensions, size_of_var};

use super::ConnectedPacket;

/// Adds a fog to the client's fog stack.
#[derive(Debug, Clone)]
pub struct UpdateFogStack<'s> {
    /// Lists of fog identifiers
    pub stack: &'s [String],
}

impl ConnectedPacket for UpdateFogStack<'_> {
    const ID: u32 = 0xa0;

    fn serialized_size(&self) -> usize {
        size_of_var(self.stack.len() as u32) +
        self.stack.iter().fold(0, |acc, f| acc + size_of_var(f.len() as u32) + f.len())
    }
}

impl Serialize for UpdateFogStack<'_> {
    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_var_u32(self.stack.len() as u32);
        for fog in self.stack {
            buffer.put_string(fog);
        }

        Ok(buffer.freeze())
    }
}
