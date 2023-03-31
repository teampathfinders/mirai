use util::{Result, Serialize, Vector};
use util::bytes::{BinaryWrite, MutableBuffer, size_of_varint};

use crate::network::ConnectedPacket;

#[derive(Debug, Clone)]
pub struct ContainerOpen {
    pub window_id: u8,
    pub container_type: u8,
    pub position: Vector<i32, 3>,
    pub container_entity_unique_id: i64,
}

impl ConnectedPacket for ContainerOpen {
    const ID: u32 = 0x2e;

    fn serialized_size(&self) -> usize {
        1 + 1 + 3 * 4 + size_of_varint(self.container_entity_unique_id)
    }
}

impl Serialize for ContainerOpen {
    fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_u8(self.window_id)?;
        buffer.write_u8(self.container_type)?;
        buffer.write_veci(&self.position)?;
        buffer.write_var_i64(self.container_entity_unique_id)
    }
}