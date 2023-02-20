use bytes::{BytesMut, Bytes};
use common::{VResult, Vector3f, WriteExtensions, size_of_var};

use common::Serialize;

use super::ConnectedPacket;

#[derive(Debug, Clone)]
pub struct SpawnExperienceOrb {
    pub position: Vector3f,
    pub amount: u32,
}

impl ConnectedPacket for SpawnExperienceOrb {
    const ID: u32 = 0x42;

    fn serialized_size(&self) -> usize {
        3 * 4 + size_of_var(self.amount)
    }
}

impl Serialize for SpawnExperienceOrb {
    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_vec3f(&self.position);
        buffer.put_var_u32(self.amount);
    }
}
