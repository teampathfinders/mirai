use bytes::{Buf, BufMut, BytesMut};
use common::{
    bail, ReadExtensions, VError, VResult, Vector3f, WriteExtensions,
};

use common::{Decodable, Encodable};

use super::GamePacket;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MovementMode {
    Normal,
    Reset,
    Teleport,
    Rotation,
}

impl TryFrom<u8> for MovementMode {
    type Error = VError;

    fn try_from(value: u8) -> VResult<Self> {
        Ok(match value {
            0 => Self::Normal,
            1 => Self::Reset,
            2 => Self::Teleport,
            3 => Self::Rotation,
            _ => bail!(BadPacket, "Invalid movement mode {value}"),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TeleportCause {
    Unknown,
    Projectile,
    ChorusFruit,
    Command,
    Behavior,
}

impl TryFrom<i32> for TeleportCause {
    type Error = VError;

    fn try_from(value: i32) -> VResult<Self> {
        Ok(match value {
            0 => Self::Unknown,
            1 => Self::Projectile,
            2 => Self::ChorusFruit,
            3 => Self::Command,
            4 => Self::Behavior,
            _ => bail!(BadPacket, "Invalid teleport cause {value}"),
        })
    }
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
    pub tick: u64,
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

impl Decodable for MovePlayer {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        let runtime_id = buffer.get_var_u64()?;
        let position = buffer.get_vec3f();
        let rotation = buffer.get_vec3f();
        let mode = MovementMode::try_from(buffer.get_u8())?;
        let on_ground = buffer.get_bool();
        let ridden_runtime_id = buffer.get_var_u64()?;

        let mut teleport_cause = TeleportCause::Unknown;
        let mut teleport_source_entity_type = 0;
        if mode == MovementMode::Teleport {
            teleport_cause = TeleportCause::try_from(buffer.get_i32())?;
            teleport_source_entity_type = buffer.get_i32();
        }

        let tick = buffer.get_var_u64()?;

        Ok(Self {
            runtime_id,
            position,
            rotation,
            mode,
            on_ground,
            ridden_runtime_id,
            teleport_cause,
            teleport_source_entity_type,
            tick,
        })
    }
}
