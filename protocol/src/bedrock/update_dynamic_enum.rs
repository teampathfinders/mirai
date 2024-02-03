use util::{BinaryWrite, size_of_varint};

use util::Serialize;

use crate::bedrock::ConnectedPacket;

/// Action to perform on the dynamic enum.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SoftEnumAction {
    Add,
    Remove,
    Set,
}

/// Updates command autocompletion entries.
#[derive(Debug, Clone)]
pub struct UpdateDynamicEnum<'a> {
    /// ID of the enum, previously specified in [`CommandEnum::enum_id`](crate::bedrock::command::CommandEnum::enum_id).
    pub enum_id: &'a str,
    /// List of enum options.
    pub options: &'a [String],
    /// Action to perform on the dynamic enum.
    pub action: SoftEnumAction,
}

impl ConnectedPacket for UpdateDynamicEnum<'_> {
    const ID: u32 = 0x72;

    fn serialized_size(&self) -> usize {
        size_of_varint(self.enum_id.len() as u32) + self.enum_id.len() +
            size_of_varint(self.options.len() as u32) +
            self.options.iter().fold(
                0, |acc, o| acc + size_of_varint(o.len() as u32) + o.len(),
            ) + 1
    }
}

impl Serialize for UpdateDynamicEnum<'_> {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_str(self.enum_id)?;
        writer.write_var_u32(self.options.len() as u32)?;
        for option in self.options {
            writer.write_str(option)?;
        }
        writer.write_u8(self.action as u8)
    }
}
