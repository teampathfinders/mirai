use bytes::{BytesMut, Bytes};
use common::{VResult, Vector3f, WriteExtensions};

use common::Serialize;

use super::ConnectedPacket;

#[derive(Debug, Clone)]
pub struct SpawnExperienceOrb {
    pub position: Vector3f,
    pub amount: u32,
}

impl ConnectedPacket for SpawnExperienceOrb {
    const ID: u32 = 0x42;
}

impl Serialize for SpawnExperienceOrb {
    fn serialize(&self) -> VResult<Bytes> {
        let mut buffer = BytesMut::with_capacity(3 * 4 + 1);

        buffer.put_vec3f(&self.position);
        buffer.put_var_u32(self.amount);

        Ok(buffer.freeze())
    }
}
