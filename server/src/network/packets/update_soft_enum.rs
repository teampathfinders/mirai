use bytes::{BufMut, BytesMut};
use common::{VResult, WriteExtensions};

use crate::network::Encodable;

use super::GamePacket;

/// Action to perform on the dynamic enum.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SoftEnumAction {
    Add,
    Remove,
    Set,
}

/// Updates command autocompletion entries.
#[derive(Debug)]
pub struct UpdateDynamicEnum {
    /// ID of the enum, previously specified in [`CommandEnum::enum_id`](super::CommandEnum::enum_id).
    pub enum_id: String,
    /// List of enum options.
    pub options: Vec<String>,
    /// Action to perform on the dynamic enum.
    pub action: SoftEnumAction,
}

impl GamePacket for UpdateDynamicEnum {
    const ID: u32 = 0x72;
}

impl Encodable for UpdateDynamicEnum {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_string(&self.enum_id);
        buffer.put_var_u32(self.options.len() as u32);
        for option in &self.options {
            buffer.put_string(option);
        }
        buffer.put_u8(self.action as u8);

        Ok(buffer)
    }
}
