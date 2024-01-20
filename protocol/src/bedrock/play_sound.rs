use util::{Result, Vector};
use util::{BinaryWrite, size_of_varint};
use util::Serialize;

use crate::bedrock::ConnectedPacket;

/// Plays a sound for the client.
#[derive(Debug)]
pub struct PlaySound<'a> {
    /// Name of the sound.
    pub name: &'a str,
    /// Position of the sound.
    pub position: Vector<i32, 3>,
    /// Volume of the sound.
    pub volume: f32,
    /// Pitch of the sound.
    pub pitch: f32,
}

impl<'a> ConnectedPacket for PlaySound<'a> {
    const ID: u32 = 0x56;

    fn serialized_size(&self) -> usize {
        size_of_varint(self.name.len() as u32) + self.name.len() +
            3 * 4 + 4 + 4
    }
}

impl<'a> Serialize for PlaySound<'a> {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_str(self.name)?;
        writer.write_veci(&self.position)?;
        writer.write_f32_le(self.volume)?;
        writer.write_f32_le(self.pitch)
    }
}
