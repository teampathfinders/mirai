use macros::variant_count;
use util::{BinaryRead};
use util::{BlockPosition, Deserialize};
use crate::bedrock::ConnectedPacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(i32)]
#[variant_count]
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
        if value <= PlayerActionType::variant_count() as i32 {
            // SAFETY: This is safe because the discriminant is in range and
            // the representations are the same. Additionally, none of the enum members
            // have a manually assigned value (this is ensured by the `variant_count` macro).
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
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let runtime_id = reader.read_var_u64()?;
        let action = PlayerActionType::try_from(reader.read_var_i32()?)?;
        let position = reader.read_block_pos()?;
        let result_position = reader.read_block_pos()?;
        let face = reader.read_var_u32()?;

        Ok(PlayerAction {
            runtime_id, action, position, result_position, face
        })
    }
}