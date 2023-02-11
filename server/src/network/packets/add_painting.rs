use bytes::BytesMut;
use common::{Vector3f, VResult, WriteExtensions};

use crate::network::Encodable;

use super::GamePacket;

#[derive(Debug, Copy, Clone)]
pub enum PaintingDirection {
    South,
    West,
    North,
    East
}

#[derive(Debug)]
pub struct AddPainting {
    pub runtime_id: u64,
    pub position: Vector3f,
    pub direction: PaintingDirection,
    pub name: String
}

impl GamePacket for AddPainting {
    const ID: u32 = 0x16;
}

impl Encodable for AddPainting {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(8 + 3 * 4 + 1 + 2 + self.name.len());

        buffer.put_var_u64(self.runtime_id); // Unique entity ID.
        buffer.put_var_u64(self.runtime_id);
        buffer.put_vec3f(&self.position);
        buffer.put_var_i32(self.direction as i32);
        buffer.put_string(&self.name);

        Ok(buffer)
    }
}