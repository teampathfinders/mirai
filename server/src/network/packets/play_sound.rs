use bytes::{BytesMut, BufMut};
use common::{VResult, Vector3f, WriteExtensions, Vector, Vector3i};

use crate::network::Encodable;

use super::GamePacket;

#[derive(Debug)]
pub struct PlaySound {
    pub name: String,
    pub position: Vector3i,
    pub volume: f32,
    pub pitch: f32
}

impl GamePacket for PlaySound {
    const ID: u32 = 0x56;
}

impl Encodable for PlaySound {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        todo!();

        // buffer.put_string(&self.name);
        // buffer.put_vec3i(&self.position);
        // buffer.put_f32(self.volume);
        // buffer.put_f32(self.pitch);

        Ok(buffer)
    }
}