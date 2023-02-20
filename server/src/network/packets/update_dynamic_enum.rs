use bytes::{BufMut, BytesMut, Bytes};
use common::{VResult, WriteExtensions, size_of_var};

use common::Serialize;

use super::ConnectedPacket;

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
    /// ID of the enum, previously specified in [`CommandEnum::enum_id`](super::CommandEnum::enum_id).
    pub enum_id: &'a str,
    /// List of enum options.
    pub options: &'a [String],
    /// Action to perform on the dynamic enum.
    pub action: SoftEnumAction,
}

impl ConnectedPacket for UpdateDynamicEnum<'_> {
    const ID: u32 = 0x72;

    fn serialized_size(&self) -> usize {
        size_of_var(self.enum_id.len() as u32) + self.enum_id.len() +
        size_of_var(self.options.len() as u32) +
        self.options.iter().fold(
            0, |acc, o| acc + size_of_var(o.len() as u32) + o.len()
        ) + 1
    }
}

impl Serialize for UpdateDynamicEnum<'_> {
    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_string(self.enum_id);
        buffer.put_var_u32(self.options.len() as u32);
        for option in self.options {
            buffer.put_string(option);
        }
        buffer.put_u8(self.action as u8);
    }
}
