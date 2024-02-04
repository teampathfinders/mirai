use macros::variant_count;
use util::{BinaryRead};
use util::{BlockPosition, Deserialize};
use crate::bedrock::ConnectedPacket;

/// Action to perform.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(i32)]
#[variant_count]
pub enum PlayerActionType {
    /// Started breaking a block.
    StartBreak,
    /// Aborted breaking a block.
    AbortBreak,
    /// Stopped breaking a block.
    StopBreak,
    /// Gets the updated state of a block.
    GetUpdatedBlock,
    /// Drops an item.
    DropItem,
    /// Starts sleeping.
    StartSleeping,
    /// Stops sleeping.
    StopSleeping,
    /// Respawns the player.
    Respawn,
    /// The player jumped.
    Jump,
    /// Starts sprinting.
    StartSprint,
    /// Stops sprinting.
    StopSprint,
    /// Starts sneaking.
    StartSneak,
    /// Stops sneaking.
    StopSneak,
    /// Destroyed a block in creative mode.
    CreativePlayerDestroyBlock,
    /// Acknowledged changing dimensions.
    DimensionChangeAcknowledgement,
    /// Starts gliding.
    StartGlide,
    /// Stops gliding.
    StopGlide,
    /// Denied building.
    BuildDenied,
    /// Something cracked has broken.
    CrackBreak,
    /// Changes the skin.
    ChangeSkin,
    /// Sets the enchantment seed.
    SetEnchantmentSeed,
    /// Starts swimming.
    StartSwimming,
    /// Stops swimming.
    StopSwimming,
    /// Starts a spin attack.
    StartSpinAttack,
    /// Stops a spin attack.
    StopSpinAttack,
    /// Interacts with a block.
    InteractBlock,
    /// Predicts a block will break.
    PredictBreak,
    /// Continues breaking a block.
    ContinueBreak,
    /// Starts using an item on something.
    StartItemUseOn,
    /// Stops using an item on something.
    StopItemUseOn,
    /// Handled a teleport.
    HandledTeleport,
    /// Missed a hit on something.
    MissedSwing,
    /// Starts crawling.
    StartCrawling,
    /// Stops crawling.
    StopCrawling,
    /// Starts flying.
    StartFlying,
    /// Stops flying.
    StopFlying,
    /// Acknowledges received server data.
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

/// Performs a player action.
#[derive(Debug)]
pub struct PlayerAction {
    /// Runtime ID of the player.
    pub runtime_id: u64,
    /// Action to perform.
    pub action: PlayerActionType,
    /// Position to perform the action at.
    pub position: BlockPosition,
    /// Position of the block the action has possibly been performed on.
    pub result_position: BlockPosition,
    /// The face of the block the action has been performed on.
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