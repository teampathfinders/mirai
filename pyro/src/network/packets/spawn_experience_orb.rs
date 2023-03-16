
use util::{Result, Vector3f};
use util::bytes::{BinaryWriter, MutableBuffer, size_of_varint};

use util::Serialize;

use crate::ConnectedPacket;

#[derive(Debug, Clone)]
pub struct SpawnExperienceOrb {
    pub position: Vector3f,
    pub amount: u32,
}

impl ConnectedPacket for SpawnExperienceOrb {
    const ID: u32 = 0x42;

    fn serialized_size(&self) -> usize {
        3 * 4 + size_of_varint(self.amount)
    }
}

impl Serialize for SpawnExperienceOrb {
    fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_vecf(&self.position);
        buffer.write_var_u32(self.amount);

        Ok(())
    }
}
