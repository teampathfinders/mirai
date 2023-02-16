use bytes::{BufMut, BytesMut};
use common::{BlockPosition, Encodable, Vector3i, VResult, WriteExtensions};
use crate::network::packets::GamePacket;

#[derive(Debug, Clone)]
pub struct ContainerOpen {
    pub window_id: u8,
    pub container_type: u8,
    pub position: Vector3i,
    pub container_entity_unique_id: i64
}

impl GamePacket for ContainerOpen {
    const ID: u32 = 0x2e;
}

impl Encodable for ContainerOpen {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_u8(self.window_id);
        buffer.put_u8(self.container_type);
        buffer.put_vec3i(&self.position);
        buffer.put_var_i64(self.container_entity_unique_id);

        Ok(buffer)
    }
}