use macros::variant_count;

use util::{Deserialize, BinaryRead, Vector, BlockPosition};

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
    PerformItemTransaction = 1 << 33,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
#[variant_count]
pub enum InputMode {
    Mouse = 1,
    Touch,
    Gamepad,
    MotionController
}

impl TryFrom<u32> for InputMode {
    type Error = anyhow::Error;

    fn try_from(v: u32) -> anyhow::Result<InputMode> {
        if v >= 1 && v <= InputMode::variant_count() as u32 {
            // SAFETY: This is safe because the discriminant is in range and
            // the representations are the same. Additionally, none of the enum members
            // have a manually assigned value (this is ensured by the `variant_count` macro).
            Ok(unsafe { std::mem::transmute::<u32, InputMode>(v) })
        } else {
            anyhow::bail!("Input mode out of range")
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(i32)]
#[variant_count]
pub enum InteractionModel {
    Touch,
    Crosshair,
    Classic
}

impl TryFrom<i32> for InteractionModel {
    type Error = anyhow::Error;

    fn try_from(v: i32) -> anyhow::Result<InteractionModel> {
        if v <= InteractionModel::variant_count() as i32 {
            // SAFETY: This is safe because the discriminant is in range and
            // the representations are the same. Additionally, none of the enum members
            // have a manually assigned value (this is ensured by the `variant_count` macro).
            Ok(unsafe { std::mem::transmute::<i32, InteractionModel>(v) })
        } else {
            anyhow::bail!("Interaction model out of range")
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum InventoryActionSource {
    Container = 0,
    World = 2,
    Creative = 3,
    Todo = 99999
}

impl TryFrom<u32> for InventoryActionSource {
    type Error = anyhow::Error;

    fn try_from(v: u32) -> anyhow::Result<Self> {
        Ok(match v {
            0 => Self::Container,
            2 => Self::World,
            3 => Self::Creative,
            99999 => Self::Todo,
            _ => anyhow::bail!("Invalid inventory action source")
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(i32)]
pub enum WindowId {
    DropContents = -100,
    Beacon = -24,
    TradingOutput = -23,
    TradingUseInputs = -22,
    TradingInput2 = -21,
    TradingInput1 = -20,
    EnchantOutput = -17,
    EnchantMaterial = -16,
    EnchantInput = -15,
    AnvilOutput = -13,
    AnvilResult = -12,
    AnvilMaterial = -11,
    ContainerInput = -10,
    CraftingUseIngredient = -5,
    CraftingResult = -4,
    CraftingRemoveIngredient = -3,
    CraftingAddIngredient = -2,
    None = -1,
    Inventory = 0,
    First = 1,
    Last = 100,
    OffHand = 119,
    Armor = 120,
    Creative = 121,
    Hotbar = 122,
    FixedInventory = 123,
    Ui = 124,
    Custom(i32)
}

impl TryFrom<i32> for WindowId {
    type Error = anyhow::Error;

    fn try_from(v: i32) -> anyhow::Result<Self> {
        Ok(match v {
            -100 => Self::DropContents,
            -24 => Self::Beacon,
            -23 => Self::TradingOutput,
            -22 => Self::TradingUseInputs,
            -21 => Self::TradingInput2,
            -20 => Self::TradingInput1,
            -17 => Self::EnchantOutput,
            -16 => Self::EnchantMaterial,
            -15 => Self::EnchantInput,
            -13 => Self::AnvilOutput,
            -12 => Self::AnvilResult,
            -11 => Self::AnvilMaterial,
            -10 => Self::ContainerInput,
            -5 => Self::CraftingUseIngredient,
            -4 => Self::CraftingResult,
            -3 => Self::CraftingRemoveIngredient,
            -2 => Self::CraftingAddIngredient,
            -1 => Self::None,
            0 => Self::Inventory,
            1 => Self::First,
            100 => Self::Last,
            119 => Self::OffHand,
            120 => Self::Armor,
            121 => Self::Creative,
            122 => Self::Hotbar,
            123 => Self::FixedInventory,
            124 => Self::Ui,
            _ => Self::Custom(v)
        })
    }
}

#[derive(Debug)]
pub struct InventoryAction {
    pub source_type: InventoryActionSource,
    pub window: Option<WindowId>,
    pub source_flags: u32,
    pub inventory_slot: u32
}

impl<'a> Deserialize<'a> for InventoryAction {
    #[allow(clippy::collection_is_never_read)]
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let source_type = InventoryActionSource::try_from(reader.read_u32_le()?)?;
        
        let mut _window = None;
        let mut _source_flags = 0;

        if source_type == InventoryActionSource::Container || source_type == InventoryActionSource::Todo {
            _window = Some(WindowId::try_from(reader.read_i8()? as i32)?);
        } else if source_type == InventoryActionSource::World {
            _source_flags = reader.read_var_u32()?;
        }

        let _inventory_slot = reader.read_var_u32()?;

        // https://github.com/Sandertv/gophertunnel/blob/36e5147307884b745b7d28d546c07ab03d4afb36/minecraft/protocol/inventory.go#L52
        todo!("Item instance reading");
    }
}

#[derive(Debug)]
pub struct LegacySetItemSlot<'a> {
    pub container: u8,
    pub slots: &'a [u8]
}

impl<'a> Deserialize<'a> for LegacySetItemSlot<'a> {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let container = reader.read_u8()?;
        let slot_count = reader.read_var_u32()?;
        let slots = reader.take_n(slot_count as usize)?;

        Ok(Self {
            container, slots
        })
    }
}

#[derive(Debug)]
pub struct TransactionData<'a> {
    pub legacy_request_id: i32,
    pub legacy_slots: Vec<LegacySetItemSlot<'a>>,
    pub actions: Vec<InventoryAction>,
    pub action_type: PlayerActionType,
    pub block_position: BlockPosition,
    pub block_face: i32,
    pub hotbar_slot: i32,
    pub position: Vector<f32, 3>,
    pub clicked_position: Vector<f32, 3>,
    pub block_runtime_id: u32
}

impl<'a> Deserialize<'a> for TransactionData<'a> {
    #[allow(clippy::collection_is_never_read)]
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let legacy_request_id = reader.read_var_i32()?;
        let mut legacy_slots = Vec::new();

        if legacy_request_id < -1 && (legacy_request_id & 1) == 0 {
            let slot_count = reader.read_var_u32()?;
            legacy_slots.reserve(slot_count as usize);

            for _ in 0..slot_count {
                legacy_slots.push(LegacySetItemSlot::deserialize_from(reader)?);
            }
        }

        let action_count = reader.read_var_u32()?;
        let mut actions = Vec::with_capacity(action_count as usize);

        for _ in 0..action_count {
            actions.push(InventoryAction::deserialize_from(reader)?);
        }

        todo!()
    }
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

    fn try_from(v: i32) -> anyhow::Result<FilterCause> {
        if v <= FilterCause::variant_count() as i32 {
            // SAFETY: This is safe because the discriminant is in range and
            // the representations are the same. Additionally, none of the enum members
            // have a manually assigned value (this is ensured by the `variant_count` macro).
            Ok(unsafe { std::mem::transmute::<i32, FilterCause>(v) })
        } else {
            anyhow::bail!("Filter cause variant out of range ({v} >= {})", Self::variant_count())
        }
    }
}

#[derive(Debug)]
pub enum ItemDescriptor<'a> {
    Invalid,
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

impl<'a> Deserialize<'a> for ItemDescriptor<'a> {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let kind = reader.read_u8()?;
        let desc = match kind {
            0 => Self::Invalid,
            1 => {
                let network_id = reader.read_i16_le()?;
                let meta = if network_id == 0 { 0 } else { reader.read_i16_le()? };

                Self::Default { network_id, meta }
            },
            2 => {
                Self::MoLang {
                    expression: reader.read_str()?,
                    version: reader.read_u8()?
                }
            },
            3 => {
                Self::ItemTag {
                    tag: reader.read_str()?
                }
            },
            4 => {
                Self::Deferred {
                    name: reader.read_str()?,
                    meta: reader.read_i16_le()?
                }
            },
            5 => {
                Self::ComplexAlias {
                    name: reader.read_str()?
                }
            }
            _ => anyhow::bail!("Item descriptor kind out of range")
        };

        Ok(desc)
    }
}

#[derive(Debug)]
pub struct ItemDescriptorCount<'a> {
    pub descriptor: ItemDescriptor<'a>,
    pub count: i32
}

impl<'a> Deserialize<'a> for ItemDescriptorCount<'a> {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let descriptor = ItemDescriptor::deserialize_from(reader)?;
        let count = reader.read_var_i32()?;

        Ok(Self {
            descriptor, count
        })
    }
}

#[derive(Debug)]
pub struct StackRequestSlotInfo {
    pub container_id: u8,
    pub slot: u8,
    pub stack_network_id: i32
}

impl<'a> Deserialize<'a> for StackRequestSlotInfo {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let container_id = reader.read_u8()?;
        let slot = reader.read_u8()?;
        let stack_network_id = reader.read_var_i32()?;

        Ok(Self {
            container_id, slot, stack_network_id
        })
    }
}

/// An action that can be performed with an item stack.
#[derive(Debug)]
pub enum StackRequestAction<'a> {
    /// Takes a certain amount of items from the given container.
    Take {
        /// Amount of items to take from the container.
        count: u8,
        /// Source slot to take the items from.
        source: StackRequestSlotInfo,
        /// Destination slot to put the items into.
        destination: StackRequestSlotInfo
    },
    /// Moves an item from one slot into another.
    Place {
        /// Amount of item to place in the slot.
        count: u8,
        /// Where to take the items from.
        source: StackRequestSlotInfo,
        /// Where to place the items.
        destination: StackRequestSlotInfo
    },
    /// Swaps two items with each other.
    Swap {
        /// Source slot to swap from.
        source: StackRequestSlotInfo,
        /// Destination slot to swap into.
        destination: StackRequestSlotInfo
    },
    /// The client dropped an item out of its inventory.
    /// [`InventoryTransaction`] is still used for items dropped from the hotbar.
    Drop {
        /// Amount of items to drop.
        count: u8,
        /// Source slot to drop from.
        source: StackRequestSlotInfo,
        /// Whether the item was dropped randomly.
        randomly: bool
    },
    /// Destroys an item when the player is in creative mode.
    Destroy {
        /// Amount of items to destroy.
        count: u8,
        /// Source slot to destroy the items from.
        source: StackRequestSlotInfo
    },
    /// An item was used to craft another item.
    Consume {
        /// Amount of items to consume in crafting.
        count: u8,
        /// Source slot to consume from.
        source: StackRequestSlotInfo
    },
    /// Used for items that are created through crafting recipe. Gophertunnel says
    /// this for example happens when empty buckets are returned to the player after
    /// crafting a cake.
    Create {
        /// Slot to put result in.
        results_slot: u8
    },
    /// No known purpose.
    PlaceInContainer {
        /// Amount of items in the stack.
        count: u8,
        /// Source slot.
        source: StackRequestSlotInfo,
        /// Destination slot.
        destination: StackRequestSlotInfo
    },
    /// No known purpose.
    TakeOutContainer {
        /// Amount of items to take out of the container.
        count: u8,
        /// Where to take the items from.
        source: StackRequestSlotInfo,
        /// Where to put the items.
        destination: StackRequestSlotInfo
    },
    /// Combines item stacks within a lab table.
    LabTableCombine,
    /// Enables a beacon using items moved to the beacon beforehand.
    BeaconPayment {
        /// The primary effect given by the beacon.
        primary_effect: i32,
        /// The secondary effect given by the beacon.
        secondary_effect: i32
    },
    /// Used when the client breaks a block.
    MineBlock {
        /// Hotbar slot that was used to break the block.
        hotbar_slot: i32,
        /// Predicted durability of the item used to break the block.
        predicted_durability: i32,
        /// Network ID of the block that was broken.
        stack_network_id: i32
    },
    /// Used when an item is crafted (or enchanted). This is sent before all other actions
    /// that happen during crafting.
    CraftRecipe {
        /// Network ID of the recipe to craft.
        recipe_network_id: u32
    },
    /// Similar to [`CraftRecipe`](`StackRequestAction::CraftRecipe`) but is sent when the client
    /// uses the recipe book instead.
    AutoCraftRecipe {
        /// Network ID of the recipe to craft.
        recipe_network_id: u32,
        /// How many times the item should be crafted.
        times_crafted: u8,
        /// Ingredients used in the recipe.
        ingredients: Vec<ItemDescriptorCount<'a>>
    },
    /// Sent when a player "crafts" an item by taking it out of the creative inventory.
    CraftCreative {
        /// Network ID of the creative item taken out of the creative inventory.
        creative_network_id: u32
    },
    /// Used when a recipe is used in an anvil. 
    /// In this case the `filter_string_index` field points to an item in the `filters` field of
    /// the [`StackRequest`] type.
    CraftRecipeOptional {
        /// Network ID of the recipe to craft.
        recipe_network_id: u32,
        /// The filter string used in this recipe.
        filter_string_index: i32
    },
    /// Sent when using a grindstone to craft something
    CraftGrindstoneRecipe {
        /// Network ID of the grindstone recipe.
        recipe_network_id: u32,
        /// Cost of the recipe.
        cost: i32
    },
    /// Sent when using a loom to craft something.
    CraftLoomRecipe {
        /// Pattern to craft with the loom.
        pattern: &'a str
    }
}   

impl<'a> Deserialize<'a> for StackRequestAction<'a> {
    fn deserialize_from<R: BinaryRead<'a>>(_reader: &mut R) -> anyhow::Result<Self> {
        todo!()
    }
}

/// A request for a change to an item stack.
#[derive(Debug)]
pub struct StackRequest<'a> {
    /// ID of the request.
    pub request_id: i32,
    /// Actions to perform.
    pub actions: Vec<StackRequestAction<'a>>,
    /// Optional filter strings used in the actions.
    pub filters: Vec<&'a str>,
    /// Reason why filtering is required.
    pub filter_cause: FilterCause
}

impl<'a> Deserialize<'a> for StackRequest<'a> {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let request_id = reader.read_var_i32()?;

        let actions_count = reader.read_var_u32()?;
        let mut actions = Vec::with_capacity(actions_count as usize);
        for _ in 0..actions_count {
            actions.push(StackRequestAction::deserialize_from(reader)?);
        }

        let filter_count = reader.read_var_u32()?;
        let mut filters = Vec::with_capacity(filter_count as usize);
        for _ in 0..filter_count {
            filters.push(reader.read_str()?);
        }

        let filter_cause = FilterCause::try_from(reader.read_i32_le()?)?;

        Ok(Self {
            request_id, actions, filters, filter_cause
        })
    }
}

macro_rules! impl_getters {
    ($($flag: ident),*) => {
        $(
            paste::paste! {
                #[inline]
                #[doc = concat!("Checks whether the `", stringify!($flag) , "` flag is set")]
                pub const fn [< $flag:snake >](&self) -> bool { self.0 & InputDataFlag::$flag as u64 != 0 }
            }
        )*
    }
}

/// Bitfield that specifies which kinds of inputs were performed in the last tick.
#[derive(Debug)]
pub struct InputData(pub u64);

impl InputData {
    impl_getters!(
        Ascend,
        Descend,
        NorthJump,
        JumpDown,
        SprintDown,
        ChangeHeight,
        Jumping,
        AutoJumpingInWater,
        Sneaking,
        SneakDown,
        Up ,
        Down ,
        Left ,
        Right ,
        UpLeft ,
        UpRight ,
        WantUp ,
        WantDown ,
        WantUpSlow ,
        Sprinting ,
        AscendBlock ,
        DescendBlock ,
        SneakToggleDown ,
        PersistSneak ,
        StartSprinting ,
        StopSprinting ,
        StartSneaking ,
        StopSneaking ,
        StartSwimming ,
        StopSwimming ,
        StartJumping ,
        StartGliding ,
        StopGliding ,
        PerformItemTransaction ,
        PerformBlockActions ,
        PerformItemStackRequest ,
        HandledTeleport ,
        Emoting ,
        MissedSwing ,
        StartCrawling ,
        StopCrawling ,
        StartFlying ,
        AcknowledgeServerData,
        StopFlying
    );
}

/// Sent every tick for server authoritative movement and inventory transactions.
#[derive(Debug)]
pub struct PlayerAuthInput<'a> {
    /// Pitch of the player.
    pub pitch: f32,
    /// Yaw of the player.
    pub yaw: f32,
    /// Yaw of the head of the player.
    pub head_yaw: f32,
    /// Position of the player.
    pub position: Vector<f32, 3>,
    /// The direction the player moved in.
    pub moved: Vector<f32, 2>,
    /// The direction the player moved in, but with analogue input.
    pub analogue_moved: Vector<f32, 2>,
    /// Bitflags specifying movement options. See [`InputData`].
    pub input_data: InputData,
    /// The controller used by the client for movement. See [`InputMode`].
    pub input_mode: InputMode,
    /// The method by which the user is playing the game. See [`PlayMode`].
    pub play_mode: PlayMode,
    /// The interaction model used by the game.
    pub interaction_model: InteractionModel,
    /// Direction the player is looking in. This seems to only be used when playing in virtual reality mode.
    pub gaze_direction: Vector<f32, 3>,
    /// The current game tick.
    pub tick: u64,
    /// Change in position compared to the previous tick.
    pub delta: Vector<f32, 3>,
    /// Item transactions that were performed in the last tick.
    pub item_transaction: Option<TransactionData<'a>>,
    /// Item stack requests that were performed in the last tick.
    pub item_stack: Option<StackRequest<'a>>,
    /// Block actions that were performed in the last tick.
    pub block_actions: Option<Vec<PlayerAction>>
}

impl ConnectedPacket for PlayerAuthInput<'_> {
    const ID: u32 = 0x90;
}

impl<'a> Deserialize<'a> for PlayerAuthInput<'a> {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let pitch = reader.read_f32_le()?;
        let yaw = reader.read_f32_le()?;
        let position = reader.read_vecf()?;
        let moved = reader.read_vecf()?;
        let head_yaw = reader.read_f32_le()?;
        let input_data = InputData(reader.read_var_u64()?);
        let input_mode = InputMode::try_from(reader.read_var_u32()?)?;
        let play_mode = PlayMode::try_from(reader.read_var_u32()?)?;
        let interaction_model = InteractionModel::try_from(reader.read_var_i32()?)?;

        let gaze_direction = if play_mode == PlayMode::VirtualReality {
            reader.read_vecf()?
        } else {
            Vector::from([0.0, 0.0, 0.0])
        };

        let tick = reader.read_var_u64()?;
        let delta = reader.read_vecf()?;

        let item_transaction = input_data.perform_item_transaction().then(|| TransactionData::deserialize_from(reader)).transpose()?;
        let item_stack = input_data.perform_item_stack_request().then(|| todo!());
        let block_actions = input_data.perform_block_actions().then(|| todo!());
        let analogue_moved = reader.read_vecf()?;
        
        Ok(Self {
            pitch, yaw, head_yaw, position, moved, analogue_moved, input_data, input_mode, play_mode,
            interaction_model, gaze_direction, tick, delta, item_transaction, item_stack, block_actions
        })
    }
}