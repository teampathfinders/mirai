use util::{BinaryRead, SharedBuffer};
use util::{BlockPosition, Deserialize};
use crate::bedrock::ConnectedPacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(i32)]
pub enum PlayerActionType {
    StartBreak,
    AbortBreak,
    StopBreak,
    GetUpdatedBlock,
    DropItem,
    StartSleeping,
    StopSleeping,
    Respawn,
    Jump,
    StartSprint,
    StopSprint,
    StartSneak,
    StopSneak,
    CreativePlayerDestroyBlock,
    DimensionChangeAcknowledgement,
    StartGlide,
    StopGlide,
    BuildDenied,
    CrackBreak,
    ChangeSkin,
    SetEnchantmentSeed,
    StartSwimming,
    StopSwimming,
    StartSpinAttack,
    StopSpinAttack,
    InteractBlock,
    PredictBreak,
    ContinueBreak,
    StartItemUseOn,
    StopItemUseOn,
    HandledTeleport,
    MissedSwing,
    StartCrawling,
    StopCrawling,
    StartFlying,
    StopFlying,
    ReceivedServerData
}

impl TryFrom<i32> for PlayerActionType {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> anyhow::Result<PlayerActionType> {
        if value <= 36 {
            Ok(unsafe {
                std::mem::transmute::<i32, PlayerActionType>(value)
            })
        } else {
            anyhow::bail!("Player action type out of range, must be <=36, is: {value}")
        }
    }
}

#[derive(Debug)]
pub struct PlayerAction {
    pub runtime_id: u64,
    pub action: PlayerActionType,
    pub position: BlockPosition,
    pub result_position: BlockPosition,
    pub face: u32
}

impl ConnectedPacket for PlayerAction {
    const ID: u32 = 0x24;
}

impl<'a> Deserialize<'a> for PlayerAction {
    fn deserialize(mut buffer: SharedBuffer<'a>) -> anyhow::Result<Self> {
        let runtime_id = buffer.read_var_u64()?;
        let action = PlayerActionType::try_from(buffer.read_var_i32()?)?;
        let position = buffer.read_block_pos()?;
        let result_position = buffer.read_block_pos()?;
        let face = buffer.read_var_u32()?;

        Ok(PlayerAction {
            runtime_id, action, position, result_position, face
        })
    }
}