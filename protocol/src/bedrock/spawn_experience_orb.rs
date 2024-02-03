use util::{Vector};
use util::{BinaryWrite, size_of_varint};
use util::Serialize;

use crate::bedrock::ConnectedPacket;

#[derive(Debug, Clone)]
pub struct SpawnExperienceOrb {
    pub position: Vector<f32, 3>,
    pub amount: u32,
}

impl ConnectedPacket for SpawnExperienceOrb {
    const ID: u32 = 0x42;

    fn serialized_size(&self) -> usize {
        3 * 4 + size_of_varint(self.amount)
    }
}

impl Serialize for SpawnExperienceOrb {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_vecf(&self.position)?;
        writer.write_var_u32(self.amount)
    }
}
