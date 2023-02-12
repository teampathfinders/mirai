use bytes::{BytesMut, BufMut};
use common::{Vector3f, VResult, WriteExtensions, VError, bail};

use crate::network::Encodable;

use super::GamePacket;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MovementMode {
    Normal,
    Reset,
    Teleport,
    Rotation
}

impl TryFrom<u8> for MovementMode {
    type Error = VError;

    fn try_from(value: u8) -> VResult<Self> {
        Ok(match value {
            0 => Self::Normal,
            1 => Self::Reset,
            2 => Self::Teleport,
            3 => Self::Rotation,
            _ => bail!(BadPacket, "Invalid movement mode")
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TeleportCause {
    Unknown,
    Projectile,
    ChorusFruit,
    Command,
    Behavior
}

#[derive(Debug)]
pub struct MovePlayer {
    pub runtime_id: u64,
    pub position: Vector3f,
    pub rotation: Vector3f,
    pub mode: MovementMode,
    pub on_ground: bool,
    pub ridden_runtime_id: u64,
    pub teleport_cause: TeleportCause,
    pub teleport_source_entity_type: i32,
    pub tick: u64
}

impl GamePacket for MovePlayer {
    const ID: u32 = 0x13;
}

impl Encodable for MovePlayer {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_var_u64(self.runtime_id);
        buffer.put_vec3f(&self.position);
        buffer.put_vec3f(&self.rotation);
        buffer.put_u8(self.mode as u8);
        buffer.put_bool(self.on_ground);
        buffer.put_var_u64(self.ridden_runtime_id);

        if self.mode == MovementMode::Teleport {
            buffer.put_i32(self.teleport_cause as i32);
            buffer.put_i32(self.teleport_source_entity_type);
        }

        buffer.put_var_u64(self.tick);

        Ok(buffer)
    }
}