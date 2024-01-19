use macros::variant_count;
use util::{Deserialize, SharedBuffer, BinaryRead, Vector, BlockPosition};
use crate::bedrock::{ConnectedPacket, PlayerActionType, PlayerAction};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u64)]
pub enum InputDataFlag {
    Ascend = 1 << 0,
    Descend = 1 << 1,
    NorthJump = 1 << 2,
    JumpDown = 1 << 3,
    SprintDown = 1 << 4,
    ChangeHeight = 1 << 5,
    Jumping = 1 << 6,
    AutoJumpingInWater = 1 << 7,
    Sneaking = 1 << 8,
    SneakDown = 1 << 9,
    Up = 1 << 10,
    Down = 1 << 11,
    Left = 1 << 12,
    Right = 1 << 13,
    UpLeft = 1 << 14,
    UpRight = 1 << 15,
    WantUp = 1 << 16,
    WantDown = 1 << 17,
    WantUpSlow = 1 << 18,
    Sprinting = 1 << 19,
    AscendBlock = 1 << 20,
    DescendBlock = 1 << 21,
    SneakToggleDown = 1 << 22,
    PersistSneak = 1 << 23,
    StartSprinting = 1 << 24,
    StopSprinting = 1 << 25,
    StartSneaking = 1 << 26,
    StopSneaking = 1 << 27,
    StartSwimming = 1 << 28,
    StopSwimming = 1 << 29,
    StartJumping = 1 << 30,
    StartGliding = 1 << 31,
    StopGliding = 1 << 32,
    PerformItemInteraction = 1 << 33,
    PerformBlockActions = 1 << 34,
    PerformItemStackRequest = 1 << 35,
    HandledTeleport = 1 << 36,
    Emoting = 1 << 37,
    MissedSwing = 1 << 38,
    StartCrawling = 1 << 39,
    StopCrawling = 1 << 40,
    StartFlying = 1 << 41,
    StopFlying = 1 << 42,
    AcknowledgeServerData = 1 << 43
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
#[variant_count]
pub enum PlayMode {
    Normal,
    Teaser,
    Screen,
    Viewer,
    VirtualReality,
    Placement,
    LivingRoom,
    ExitLevel,
    ExitLevelLivingRoom
}

impl TryFrom<u32> for PlayMode {
    type Error = anyhow::Error;
    
    fn try_from(v: u32) -> anyhow::Result<Self> {
        if v <= Self::variant_count() as u32 {
            // SAFETY: This is safe because the enum has a `u32` repr and the discriminant is in range.
            Ok(unsafe { std::mem::transmute::<u32, Self>(v) })
        } else {
            anyhow::bail!("Play mode out of range ({v} > {})", Self::variant_count());
        }
    }
}

#[derive(Debug)]
pub struct TransactionData {
    pub legacy_request_id: i32,
    pub legacy_slots: Vec<LegacyItemSlot>,
    pub actions: Vec<InventoryAction>,
    pub action_type: PlayerActionType,
    pub block_position: BlockPosition,
    pub block_face: i32,
    pub hotbar_slot: i32,
    pub position: Vector<f32, 3>,
    pub clicked_position: Vector<f32, 3>,
    pub block_runtime_id: u32
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(i32)]
#[variant_count]
pub enum FilterCause {
    ServerChatPublic,
    ServerChatWhisper,
    SignText,
    AnvilText,
    BookAndQuillText,
    CommandBlockText,
    BlockActorDataText,
    JoinEventText,
    LeaveEventText,
    SlashCommandChat,
    CartographyText,
    KickCommand,
    TitleCommand,
    SummonCommand
}

impl TryFrom<i32> for FilterCause {
    type Error = anyhow::Error;

    fn try_from(v: i32) -> anyhow::Result<Self> {
        if v <= Self::variant_count() {
            Ok(unsafe { std::mem::transmute::<i32, Self>(v) })
        } else {
            anyhow::bail!("Filter cause variant out of range ({v} >= {})", Self::variant_count())
        }
    }
}

#[derive(Debug)]
pub enum ItemDescriptor<'a> {
    Default {
        network_id: i16,
        meta: i16
    },
    MoLang {
        expression: &'a str,
        version: u8
    },
    ItemTag {
        tag: &'a str
    },
    Deferred {
        name: &'a str,
        meta: i16
    },
    ComplexAlias {
        name: &'a str
    }
}

#[derive(Debug)]
pub struct ItemDescriptorCount<'a> {
    pub descriptor: ItemDescriptor<'a>,
    pub count: i32
}

#[derive(Debug)]
pub struct StackRequestSlotInfo {
    pub container_id: u8,
    pub slot: u8,
    pub stack_network_id: i32
}

#[derive(Debug)]
pub enum StackRequestAction<'a> {
    /// Takes a certain amount of items from the given container.
    Take {
        count: u8,
        source: StackRequestSlotInfo,
        destination: StackRequestSlotInfo
    },
    /// Moves an item from one slot into another.
    Place {
        count: u8,
        source: StackRequestSlotInfo,
        destination: StackRequestSlotInfo
    },
    /// Swaps two items with each other.
    Swap {
        source: StackRequestSlotInfo,
        destination: StackRequestSlotInfo
    },
    /// The client dropped an item out of its inventory.
    /// [`InventoryTransaction`] is still used for items dropped from the hotbar.
    Drop {
        count: u8,
        source: StackRequestSlotInfo,
        randomly: bool
    },
    /// Destroys an item when the player is in creative mode.
    Destroy {
        count: u8,
        source: StackRequestSlotInfo
    },
    /// An item was used to craft another item.
    Consume {
        count: u8,
        source: StackRequestSlotInfo
    },
    /// Used for items that are created through crafting recipe. Gophertunnel says
    /// this for example happens when empty buckets are returned to the player after
    /// crafting a cake.
    Create {
        results_slot: u8
    },
    /// No known purpose.
    PlaceInContainer {
        count: u8,
        source: StackRequestSlotInfo,
        destination: StackRequestSlotInfo
    },
    /// No known purpose.
    TakeOutContainer {
        count: u8,
        source: StackRequestSlotInfo,
        destination: StackRequestSlotInfo
    },
    /// Combines item stacks within a lab table.
    LabTableCombine,
    /// Enables a beacon using items moved to the beacon beforehand.
    BeaconPayment {
        primary_effect: i32,
        secondary_effect: i32
    },
    /// Used when the client breaks a block.
    MineBlock {
        hotbar_slot: i32,
        predicted_durability: i32,
        stack_network_id: i32
    },
    /// Used when an item is crafted (or enchanted). This is sent before all other actions
    /// that happen during crafting.
    CraftRecipe {
        recipe_network_id: u32
    },
    /// Similar to [`CraftRecipe`](`StackRequestAction::CraftRecipe`) but is sent when the client
    /// uses the recipe book instead.
    AutoCraftRecipe {
        recipe_network_id: u32,
        times_crafted: u8,
        ingredients: Vec<ItemDescriptorCount<'a>>
    },
    /// Sent when a player "crafts" an item by taking it out of the creative inventory.
    CraftCreative {
        creative_network_id: u32
    },
    /// Used when a recipe is used in an anvil. 
    /// In this case the `filter_string_index` field points to an item in the `filters` field of
    /// the [`StackRequest`] type.
    CraftRecipeOptional {
        recipe_network_id: u32,
        filter_string_index: i32
    },
    /// Sent when using a grindstone to craft something
    CraftGrindstoneRecipe {
        recipe_network_id: u32,
        cost: i32
    },
    CraftLoomRecipe {
        pattern: &'a str
    }
}   

impl<'a> StackRequestAction<'a> {
    pub fn deserialize(buffer: &mut SharedBuffer<'a>) -> anyhow::Result<Self> {
        todo!()
    }
}

#[derive(Debug)]
pub struct StackRequest<'a> {
    pub request_id: i32,
    pub actions: Vec<StackRequestAction>,
    pub filters: Vec<&'a str>,
    pub filter_cause: FilterCause
}

impl<'a> StackRequest<'a> {
    pub fn deserialize(buffer: &mut SharedBuffer<'a>) -> anyhow::Result<Self> {
        let request_id = buffer.read_var_i32()?;

        let actions_count = buffer.read_var_u32()?;
        let mut actions = Vec::with_capacity(actions_count as usize);
        for _ in 0..actions_count {
            actions.push(StackRequestAction::deserialize(buffer)?);
        }

        let mut filter_count = buffer.read_var_u32()?;
        let mut filters = Vec::with_capacity(filter_count as usize);
        for _ in 0..filter_count {
            filters.push(buffer.read_str()?);
        }

        let filter_cause = FilterCause::try_from(buffer.read_i32_le()?)?;

        Ok(Self {
            request_id, actions, filters, filter_cause
        })
    }
}

#[derive(Debug)]
pub struct PlayerAuthInput<'a> {
    pub pitch: f32,
    pub yaw: f32,
    pub head_yaw: f32,

    pub position: Vector<f32, 3>,
    pub moved: Vector<f32, 2>,
    pub analogue_moved: Vector<f32, 2>,

    pub input_data: u64,
    pub input_mode: u32,
    pub play_mode: u32,
    pub interaction_model: i32,
    pub gaze_direction: Vector<f32, 3>,

    pub tick: u64,
    pub delta: Vector<f32, 3>,

    pub item_transaction: TransactionData,
    pub item_stack: StackRequest<'a>,
    pub block_actions: Vec<PlayerAction>
}

impl ConnectedPacket for PlayerAuthInput<'_> {
    const ID: u32 = 0x90;
}

impl<'a> Deserialize<'a> for PlayerAuthInput<'a> {
    fn deserialize(mut buffer: SharedBuffer<'a>) -> anyhow::Result<Self> {
        let pitch = buffer.read_f32_le()?;
        let yaw = buffer.read_f32_le()?;
        let position = buffer.read_vecf()?;
        let moved = buffer.read_vecf()?;
        let head_yaw = buffer.read_f32_le()?;
        let input_data = buffer.read_var_u64()?;
        let input_mode = buffer.read_var_u32()?;
        let play_mode = PlayMode::try_from(buffer.read_var_u32()?)?;
        let interaction_model = buffer.read_var_i32()?;

        let gaze_direction = if play_mode == PlayMode::VirtualReality {
            buffer.read_vecf()?
        } else {
            Vector::from([0.0, 0.0, 0.0])
        };

        let tick = buffer.read_var_u64()?;
        let delta = buffer.read_vecf()?;

        if input_data & InputDataFlag::PerformItemInteraction != 0 {
            todo!();
        }

        if input_data & InputDataFlag::PerformItemStackRequest != 0 {
            todo!();
        }

        if input_data & InputDataFlag::PerformBlockActions != 0 {

        }

        let analogue_moved = buffer.read_vecf()?;
        
        Ok(Self {
            pitch, yaw, head_yaw, position, moved, analogue_moved, input_data, input_mode, play_mode,
            interaction_model, gaze_direction, tick, delta, item_transaction, item_stack, block_actions
        })
    }
}