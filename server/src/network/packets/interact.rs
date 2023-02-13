use bytes::{Buf, BytesMut};
use common::{bail, ReadExtensions, VError, VResult, Vector3f};

use common::Decodable;

use super::GamePacket;

/// All types of interaction.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InteractAction {
    LeaveVehicle = 3,
    MouseOverEntity = 4,
    NpcOpen = 5,
    OpenInventory = 6,
}

impl TryFrom<u8> for InteractAction {
    type Error = VError;

    fn try_from(value: u8) -> VResult<Self> {
        Ok(match value {
            3 => Self::LeaveVehicle,
            4 => Self::MouseOverEntity,
            5 => Self::NpcOpen,
            6 => Self::OpenInventory,
            _ => bail!(BadPacket, "Invalid interact action type"),
        })
    }
}

#[derive(Debug)]
pub struct Interact {
    /// Type of action to perform.
    pub action: InteractAction,
    /// Target of the interaction.
    pub target_runtime_id: u64,
    /// Position of the interaction,
    pub position: Vector3f,
}

impl GamePacket for Interact {
    const ID: u32 = 0x21;
}

impl Decodable for Interact {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        let action = InteractAction::try_from(buffer.get_u8())?;
        let target_runtime_id = buffer.get_var_u64()?;

        let position = match action {
            InteractAction::MouseOverEntity | InteractAction::LeaveVehicle => {
                buffer.get_vec3f()
            }
            _ => Vector3f::from([0.0, 0.0, 0.0]),
        };

        Ok(Self {
            action,
            target_runtime_id,
            position,
        })
    }
}
