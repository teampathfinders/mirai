use bytes::{BufMut, BytesMut, Bytes};
use util::{BlockPosition, Serialize, Vector3i, Result, WriteExtensions, size_of_varint};
use crate::network::packets::ConnectedPacket;

#[derive(Debug, Clone)]
pub struct ContainerOpen {
    pub window_id: u8,
    pub container_type: u8,
    pub position: Vector3i,
    pub container_entity_unique_id: i64
}

impl ConnectedPacket for ContainerOpen {
    const ID: u32 = 0x2e;

    fn serialized_size(&self) -> usize {
        1 + 1 + 3 * 4 + size_of_varint(self.container_entity_unique_id)
    }
}

impl Serialize for ContainerOpen {
    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.write_le::<u8>(self.window_id);
        buffer.write_le::<u8>(self.container_type);
        buffer.put_vec3i(&self.position);
        buffer.put_var_i64(self.container_entity_unique_id);
    }
}