
use util::{Result, Vector};
use util::bytes::{BinaryWrite, MutableBuffer, size_of_varint};

use util::Serialize;

use crate::ConnectedPacket;

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
    fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_str(self.name)?;
        buffer.write_veci(&self.position)?;
        buffer.write_f32_le(self.volume)?;
        buffer.write_f32_le(self.pitch)
    }
}
