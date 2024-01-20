use util::{Result, Serialize};
use util::{BinaryWrite, MutableBuffer, size_of_varint};

use crate::bedrock::ConnectedPacket;

/// Adds a fog to the client's fog stack.
#[derive(Debug, Clone)]
pub struct UpdateFogStack<'s> {
    /// Lists of fog identifiers
    pub stack: &'s [String],
}

impl ConnectedPacket for UpdateFogStack<'_> {
    const ID: u32 = 0xa0;

    fn serialized_size(&self) -> usize {
        size_of_varint(self.stack.len() as u32) +
            self.stack.iter().fold(0, |acc, f| acc + size_of_varint(f.len() as u32) + f.len())
    }
}

impl Serialize for UpdateFogStack<'_> {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_u32(self.stack.len() as u32)?;
        for fog in self.stack {
            writer.write_str(fog)?;
        }

        Ok(())
    }
}
