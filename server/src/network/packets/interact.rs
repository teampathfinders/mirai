use bytes::{BytesMut, Buf};
use common::{Vector3f, VResult, VError, bail, ReadExtensions};

use crate::network::Decodable;

use super::GamePacket;

#[derive(Debug, Copy, Clone)]
pub enum ActionType {
    LeaveVehicle = 3,
    MouseOverEntity = 4,
    NpcOpen = 5,
    OpenInventory = 6
}

impl TryFrom<u8> for ActionType {
    type Error = VError;

    fn try_from(value: u8) -> VResult<Self> {
        Ok(match value {
            3 => Self::LeaveVehicle,
            4 => Self::MouseOverEntity,
            5 => Self::NpcOpen,
            6 => Self::OpenInventory,
            _ => bail!(BadPacket, "Invalid interact action type")
        })
    }
}

#[derive(Debug)]
pub struct Interact {
    /// Type of action to perform.
    pub action: ActionType,
    /// Target of the interaction.
    pub target_runtime_id: u64,
    /// Position of the interaction,
    pub position: Vector3f
}

impl GamePacket for Interact {
    const ID: u32 = 0x21;
}

impl Decodable for Interact {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        let action = ActionType::try_from(buffer.get_u8())?;
        let target_runtime_id = buffer.get_var_u64()?;
        let position = buffer.get_vec3f();

        Ok(Self {
            action, target_runtime_id, position
        })
    }
}