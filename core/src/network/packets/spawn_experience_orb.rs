use util::{Result, Vector};
use util::bytes::{BinaryWrite, MutableBuffer, size_of_varint};
use util::Serialize;

use crate::network::ConnectedPacket;

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
    fn serialize<W>(&self, buffer: W) -> anyhow::Result<()> where W: BinaryWrite {
        buffer.write_vecf(&self.position)?;
        buffer.write_var_u32(self.amount)
    }
}
