use bytes::{BufMut, BytesMut, Bytes};
use util::{Result, Vector, Vector3f, Vector3i};
use util::bytes::{BinaryWriter, MutableBuffer, size_of_varint};

use util::Serialize;

use super::ConnectedPacket;

/// Plays a sound for the client.
#[derive(Debug)]
pub struct PlaySound<'s> {
    /// Name of the sound.
    pub name: &'s str,
    /// Position of the sound.
    pub position: Vector3i,
    /// Volume of the sound.
    pub volume: f32,
    /// Pitch of the sound.
    pub pitch: f32,
}

impl<'s> ConnectedPacket for PlaySound<'s> {
    const ID: u32 = 0x56;

    fn serialized_size(&self) -> usize {
        size_of_varint(self.name.len() as u32) + self.name.len() +
            3 * 4 + 4 + 4
    }
}

impl<'s> Serialize for PlaySound<'s> {
    fn serialize(&self, buffer: &mut MutableBuffer) {
        buffer.write_str(self.name);
        buffer.write_vec3i(&self.position);
        buffer.write_f32_le(self.volume);
        buffer.write_f32_le(self.pitch);
    }
}
