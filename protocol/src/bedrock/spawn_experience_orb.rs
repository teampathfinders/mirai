use util::{Vector};
use util::{BinaryWrite, size_of_varint};
use util::Serialize;

use crate::bedrock::ConnectedPacket;

/// Spawns an experience orb.
/// 
/// Orbs cannot be spawned with the standard entity packets.
#[derive(Debug, Clone)]
pub struct SpawnExperienceOrb {
    /// Position of the orb.
    pub position: Vector<f32, 3>,
    /// Amount of experience this orb has.
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
