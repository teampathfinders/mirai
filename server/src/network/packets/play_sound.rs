use bytes::{BufMut, BytesMut};
use common::{VResult, Vector, Vector3f, Vector3i, WriteExtensions};

use common::Encodable;

use super::GamePacket;

/// Plays a sound for the client.
#[derive(Debug, Clone)]
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

impl GamePacket for PlaySound<'_> {
    const ID: u32 = 0x56;
}

impl Encodable for PlaySound<'_> {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_string(&self.name);
        buffer.put_vec3i(&self.position);
        buffer.put_f32_le(self.volume);
        buffer.put_f32_le(self.pitch);

        Ok(buffer)
    }
}
