use bytes::BytesMut;
use common::{VResult, Vector3f, WriteExtensions};

use crate::network::Encodable;

use super::GamePacket;

#[derive(Debug)]
pub struct SpawnExperienceOrb {
    pub position: Vector3f,
    pub amount: u32,
}

impl GamePacket for SpawnExperienceOrb {
    const ID: u32 = 0x42;
}

impl Encodable for SpawnExperienceOrb {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(3 * 4 + 1);

        buffer.put_vec3f(&self.position);
        buffer.put_var_u32(self.amount);

        Ok(buffer)
    }
}
