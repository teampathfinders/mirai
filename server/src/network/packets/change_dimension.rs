use bytes::BytesMut;
use common::{Vector3f, VResult, WriteExtensions};

use crate::network::Encodable;

use super::GamePacket;

#[derive(Debug, Copy, Clone)]
pub enum Dimension {
    Overworld,
    Nether,
    End,
}

#[derive(Debug)]
pub struct ChangeDimension {
    pub dimension: Dimension,
    pub position: Vector3f,
    pub respawn: bool
}

impl GamePacket for ChangeDimension {
    const ID: u32 = 0x3d;
}

impl Encodable for ChangeDimension {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(1 + 3 * 4 + 1);

        buffer.put_var_i32(self.dimension as i32);
        buffer.put_vec3f(&self.position);
        buffer.put_bool(self.respawn);

        Ok(buffer)
    }
}