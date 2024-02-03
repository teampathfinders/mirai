use util::{bail, Vector};
use util::{Deserialize, Serialize};
use util::{BinaryRead, BinaryWrite, size_of_varint};

use crate::bedrock::ConnectedPacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MovementMode {
    Normal,
    Reset,
    Teleport,
    Rotation,
}

impl TryFrom<u8> for MovementMode {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> anyhow::Result<Self> {
        Ok(match value {
            0 => Self::Normal,
            1 => Self::Reset,
            2 => Self::Teleport,
            3 => Self::Rotation,
            _ => bail!(Malformed, "Invalid movement mode {value}"),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TeleportCause {
    Unknown,
    Projectile,
    ChorusFruit,
    Command,
    Behavior,
}

impl TryFrom<i32> for TeleportCause {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> anyhow::Result<Self> {
        Ok(match value {
            0 => Self::Unknown,
            1 => Self::Projectile,
            2 => Self::ChorusFruit,
            3 => Self::Command,
            4 => Self::Behavior,
            _ => bail!(Malformed, "Invalid teleport cause {value}"),
        })
    }
}

#[derive(Debug, Clone)]
pub struct MovePlayer {
    pub runtime_id: u64,
    pub translation: Vector<f32, 3>,
    pub pitch: f32,
    pub yaw: f32,
    pub head_yaw: f32,
    pub mode: MovementMode,
    pub on_ground: bool,
    pub ridden_runtime_id: u64,
    pub teleport_cause: TeleportCause,
    pub teleport_source_type: i32,
    pub tick: u64,
}

impl ConnectedPacket for MovePlayer {
    const ID: u32 = 0x13;

    fn serialized_size(&self) -> usize {
        size_of_varint(self.runtime_id) +
            3 * 4 + 3 * 4 + 1 + 1 +
            size_of_varint(self.ridden_runtime_id) +
            size_of_varint(self.tick) +
            if self.mode == MovementMode::Teleport {
                4 + 4
            } else {
                0
            }
    }
}

impl Serialize for MovePlayer {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_u64(self.runtime_id)?;
        writer.write_vecf(&self.translation)?;
        writer.write_f32_le(self.pitch)?;
        writer.write_f32_le(self.yaw)?;
        writer.write_f32_le(self.head_yaw)?;
        writer.write_u8(self.mode as u8)?;
        writer.write_bool(self.on_ground)?;
        writer.write_var_u64(self.ridden_runtime_id)?;

        if self.mode == MovementMode::Teleport {
            writer.write_i32_be(self.teleport_cause as i32)?;
            writer.write_i32_be(self.teleport_source_type)?;
        }

        writer.write_var_u64(self.tick)
    }
}

impl<'a> Deserialize<'a> for MovePlayer {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let runtime_id = reader.read_var_u64()?;
        let position = reader.read_vecf()?;
        let pitch = reader.read_f32_le()?;
        let yaw = reader.read_f32_le()?;
        let head_yaw = reader.read_f32_le()?;
        let mode = MovementMode::try_from(reader.read_u8()?)?;
        let on_ground = reader.read_bool()?;
        let ridden_runtime_id = reader.read_var_u64()?;

        let (teleport_cause, teleport_source_type) = if mode == MovementMode::Teleport {
            let cause = TeleportCause::try_from(reader.read_i32_be()?)?;
            let source_type = reader.read_i32_be()?;

            (cause, source_type)
        } else {
            (TeleportCause::Unknown, 0)
        };

        let tick = reader.read_var_u64()?;

        Ok(Self {
            runtime_id,
            translation: position,
            yaw, pitch, head_yaw,
            mode,
            on_ground,
            ridden_runtime_id,
            teleport_cause,
            teleport_source_type,
            tick,
        })
    }
}
