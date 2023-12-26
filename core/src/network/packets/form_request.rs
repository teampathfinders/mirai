use util::bytes::{BinaryWrite, MutableBuffer, size_of_string, size_of_varint};
use util::Serialize;
use crate::network::ConnectedPacket;

#[derive(Debug, Clone)]
pub struct FormRequest<'a> {
    pub id: u32,
    pub data: &'a str
}

impl<'a> ConnectedPacket for FormRequest<'a> {
    const ID: u32 = 0x64;

    fn serialized_size(&self) -> usize {
        size_of_varint(self.id) + size_of_string(self.data)
    }
}

impl<'a> Serialize for FormRequest<'a> {
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_var_u32(self.id)?;
        buffer.write_str(self.data)
    }
}