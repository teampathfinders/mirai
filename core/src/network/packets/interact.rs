use util::{bail, Error, Result, Vector};
use util::bytes::{BinaryRead, SharedBuffer};
use util::Deserialize;

use crate::network::ConnectedPacket;

/// All types of interaction.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InteractAction {
    LeaveVehicle = 3,
    MouseOverEntity = 4,
    NpcOpen = 5,
    OpenInventory = 6,
}

impl TryFrom<u8> for InteractAction {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> anyhow::Result<Self> {
        Ok(match value {
            3 => Self::LeaveVehicle,
            4 => Self::MouseOverEntity,
            5 => Self::NpcOpen,
            6 => Self::OpenInventory,
            _ => bail!(Malformed, "Invalid interact action type"),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Interact {
    /// Type of action to perform.
    pub action: InteractAction,
    /// Target of the interaction.
    pub target_runtime_id: u64,
    /// Position of the interaction,
    pub position: Vector<f32, 3>,
}

impl ConnectedPacket for Interact {
    const ID: u32 = 0x21;
}

impl<'a> Deserialize<'a> for Interact {
    fn deserialize<R>(reader: R) -> anyhow::Result<Self>
    where
        R: BinaryRead<'a> + 'a
    {
        let action = InteractAction::try_from(reader.read_u8()?)?;
        let target_runtime_id = reader.read_var_u64()?;

        let position = match action {
            InteractAction::MouseOverEntity | InteractAction::LeaveVehicle => {
                reader.read_vecf()?
            }
            _ => Vector::from([0.0, 0.0, 0.0]),
        };

        Ok(Self { action, target_runtime_id, position })
    }
}
