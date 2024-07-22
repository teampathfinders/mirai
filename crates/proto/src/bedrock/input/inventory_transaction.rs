use std::{collections::HashMap, sync::atomic::{AtomicI32, Ordering}};

use util::{BinaryRead, BinaryWrite, BlockPosition, Deserialize, RVec, Serialize, Vector};

use crate::bedrock::ConnectedPacket;

use super::WindowId;

pub static SHIELD_ID: AtomicI32 = AtomicI32::new(0);

#[derive(Debug, Clone)]
pub struct LegacyTransactionEntry<'a> {
    pub container_id: u8,
    pub changed_slots: &'a [u8]
}

impl<'a> Serialize for LegacyTransactionEntry<'a> {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_u8(self.container_id)?;
        writer.write_var_u32(self.changed_slots.len() as u32)?;
        writer.write_all(self.changed_slots)?;

        Ok(())
    }
}

impl<'a> Deserialize<'a> for LegacyTransactionEntry<'a> {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let container_id = reader.read_u8()?;
        let changed_slots_len = reader.read_var_u32()?;
        let changed_slots = reader.take_n(changed_slots_len as usize)?;

        Ok(LegacyTransactionEntry {
            container_id, changed_slots
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum TransactionSourceType {
    Container {
        inventory_id: WindowId
    } = 0,
    Global = 1,
    WorldInteraction {
        flags: u32
    } = 2,
    Creative = 3,
    CraftSlot {
        action: u32
    } = 100,
    Craft {
        inventory_id: WindowId,
        action: u32
    } = 99_999
}

impl TransactionSourceType {
    pub const fn as_id(&self) -> u32 {
        match self {
            Self::Container { .. } => 0,
            Self::Global => 1,
            Self::WorldInteraction { .. } => 2,
            Self::Creative => 3,
            Self::CraftSlot { .. } => 100,
            Self::Craft { .. } => 99_999
        }
    }
}

impl Serialize for TransactionSourceType {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_u32(self.as_id())?;

        match self {
            Self::Container { inventory_id } => {
                writer.write_var_i32(Into::<i32>::into(*inventory_id))?;
            },
            Self::WorldInteraction { flags } => {
                writer.write_var_u32(*flags)?;
            },
            Self::CraftSlot { action } => {
                writer.write_var_u32(*action)?;
            },
            Self::Craft { inventory_id, action } => {
                writer.write_var_i32(Into::<i32>::into(*inventory_id))?;
                writer.write_var_u32(*action)?;
            },
            _ => {}
        }

        Ok(())
    }
}

impl<'a> Deserialize<'a> for TransactionSourceType {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let source_type = reader.read_var_u32()?;

        Ok(match source_type {
            0 => Self::Container {
                inventory_id: WindowId::try_from(reader.read_var_i32()?)?
            },
            1 => Self::Global,
            2 => Self::WorldInteraction {
                flags: reader.read_var_u32()?
            },
            3 => Self::Creative,
            100 => Self::CraftSlot {
                action: reader.read_var_u32()?
            },
            99_999 => Self::Craft {
                inventory_id: WindowId::try_from(reader.read_var_i32()?)?,
                action: reader.read_var_u32()?
            },
            _ => anyhow::bail!("Invalid transaction source type")
        })
    }
}

#[derive(Debug, Clone)]
pub struct TransactionAction<'a> {
    pub source_type: TransactionSourceType,
    pub slot: u32,
    pub old_item: ItemInstance<'a>,
    pub new_item: ItemInstance<'a>
}

impl<'a> Serialize for TransactionAction<'a> {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        self.source_type.serialize_into(writer)?;
        writer.write_var_u32(self.slot)?;
        self.old_item.serialize_into(writer)?;
        self.new_item.serialize_into(writer)
    }
}

impl<'a> Deserialize<'a> for TransactionAction<'a> {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let source_type = TransactionSourceType::deserialize_from(reader)?;
        // tracing::debug!("Source type {source_type:?}");
        let slot = reader.read_var_u32()?;
        // tracing::debug!("Slot {slot}");

        let old_item = ItemInstance::deserialize_from(reader)?;
        // tracing::debug!("Old item: {old_item:?}");
        let new_item = ItemInstance::deserialize_from(reader)?;
        // tracing::debug!("New item: {new_item:?}");

        Ok(TransactionAction {
            source_type,
            slot,
            old_item,
            new_item
        })
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum UseItemAction {
    #[default]
    ClickBlock,
    ClickAir,
    BreakBlock
}

impl Into<u32> for UseItemAction {
    fn into(self) -> u32 {
        match self {
            Self::ClickBlock => 0,
            Self::ClickAir => 1,
            Self::BreakBlock => 2
        }
    }
}

impl TryFrom<u32> for UseItemAction {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> anyhow::Result<UseItemAction> {
        Ok(match value {
            0 => Self::ClickBlock,
            1 => Self::ClickAir,
            2 => Self::BreakBlock,
            _ => anyhow::bail!("Invalid use item action")
        })
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum UseOnEntityAction {
    #[default]
    Interact,
    Attack
}

impl Into<u32> for UseOnEntityAction {
    fn into(self) -> u32 {
        match self {
            Self::Interact => 0,
            Self::Attack => 1
        }
    }
}

impl TryFrom<u32> for UseOnEntityAction {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> anyhow::Result<UseOnEntityAction> {
        Ok(match value {
            0 => Self::Interact,
            1 => Self::Attack,
            _ => anyhow::bail!("Invalid use on entity action")
        })
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum ReleaseAction {
    #[default]
    Release,
    Consume
}

impl Into<u32> for ReleaseAction {
    fn into(self) -> u32 {
        match self {
            Self::Release => 0,
            Self::Consume => 1
        }
    }
}

impl TryFrom<u32> for ReleaseAction {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> anyhow::Result<ReleaseAction> {
        Ok(match value {
            0 => Self::Release,
            1 => Self::Consume,
            _ => anyhow::bail!("Invalid release action")
        })
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum TransactionType<'a> {
    #[default]
    Normal,
    Mismatch,
    Use {
        action_type: UseItemAction,
        block_position: BlockPosition,
        face: i32,
        hotbar_slot: i32,
        held_item: ItemInstance<'a>,
        player_position: Vector<f32, 3>,
        click_position: Vector<f32, 3>,
        block_runtime_id: u32
    },
    UseOnEntity {
        entity_runtime_id: u64,
        action_type: UseOnEntityAction,
        hotbar_slot: i32,
        held_item: ItemInstance<'a>,
        player_position: Vector<f32, 3>,
        click_position: Vector<f32, 3>
    },
    Release {
        action_type: ReleaseAction,
        hotbar_slot: i32,
        held_item: ItemInstance<'a>,
        head_position: Vector<f32, 3>
    }
}

impl<'a> TransactionType<'a> {
    pub const fn as_id(&self) -> u32 {
        match self {
            Self::Normal => 0,
            Self::Mismatch => 1,
            Self::Use { .. } => 2,
            Self::UseOnEntity { .. } => 3,
            Self::Release { .. } => 4
        }
    }

    pub fn deserialize_from<R: BinaryRead<'a>>(transaction_type: u32, reader: &mut R) -> anyhow::Result<TransactionType<'a>> {
        Ok(match transaction_type {
            0 => Self::Normal,
            1 => Self::Mismatch,
            2 => Self::Use {
                action_type: UseItemAction::try_from(reader.read_var_u32()?)?,
                block_position: reader.read_block_pos()?,
                face: reader.read_i32_le()?,
                hotbar_slot: reader.read_i32_le()?,
                held_item: ItemInstance::deserialize_from(reader)?,
                player_position: reader.read_vecf()?,
                click_position: reader.read_vecf()?,
                block_runtime_id: reader.read_var_u32()?
            },
            3 => Self::UseOnEntity {
                entity_runtime_id: reader.read_var_u64()?,
                action_type: UseOnEntityAction::try_from(reader.read_var_u32()?)?,
                hotbar_slot: reader.read_i32_le()?,
                held_item: ItemInstance::deserialize_from(reader)?,
                player_position: reader.read_vecf()?,
                click_position: reader.read_vecf()?
            },
            4 => Self::Release {
                action_type: ReleaseAction::try_from(reader.read_var_u32()?)?,
                hotbar_slot: reader.read_i32_le()?,
                held_item: ItemInstance::deserialize_from(reader)?,
                head_position: reader.read_vecf()?
            },
            _ => anyhow::bail!("Invalid transaction type")
        })
    }
}

impl<'a> Serialize for TransactionType<'a> {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        match self {
            Self::Normal |
            Self::Mismatch => Ok(()),
            Self::Use {
                action_type,
                block_position,
                face,
                hotbar_slot,
                held_item,
                player_position,
                click_position,
                block_runtime_id
            } => {
                writer.write_var_u32(Into::<u32>::into(*action_type))?;
                writer.write_block_pos(block_position)?;
                writer.write_i32_le(*face)?;
                writer.write_i32_le(*hotbar_slot)?;
                held_item.serialize_into(writer)?;
                writer.write_vecf(player_position)?;
                writer.write_vecf(click_position)?;
                writer.write_var_u32(*block_runtime_id)
            }
            Self::UseOnEntity { 
                entity_runtime_id, 
                action_type, 
                hotbar_slot, 
                held_item, 
                player_position, 
                click_position 
            } => {
                writer.write_var_u64(*entity_runtime_id)?;
                writer.write_var_u32(Into::<u32>::into(*action_type))?;
                writer.write_i32_le(*hotbar_slot)?;
                held_item.serialize_into(writer)?;
                writer.write_vecf(player_position)?;
                writer.write_vecf(click_position)
            }
            Self::Release {
                action_type,
                hotbar_slot,
                held_item,
                head_position
            } => {
                writer.write_var_u32(Into::<u32>::into(*action_type))?;
                writer.write_i32_le(*hotbar_slot)?;
                held_item.serialize_into(writer)?;
                writer.write_vecf(head_position)
            }
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ItemInstance<'a> {
    pub network_id: i32,
    pub count: u16,
    pub metadata: u32,
    pub stack_id: Option<i32>,
    pub block_runtime_id: i32,
    pub nbt: HashMap<String, nbt::Value>,
    pub can_place_on: Vec<&'a str>,
    pub can_destroy: Vec<&'a str>,
    pub blocking_tick: i64
}

impl<'a> ItemInstance<'a> {
    /// This is a function instead of a constant due to the
    /// non-constness of HashMap::new.
    #[inline(always)]
    pub fn air() -> ItemInstance<'a> {
        ItemInstance {
            network_id: 0,
            count: 0,
            metadata: 0,
            stack_id: None,
            block_runtime_id: 0,
            nbt: HashMap::new(),
            can_place_on: vec![],
            can_destroy: vec![],
            blocking_tick: 0
        }
    }
}

impl<'a> Serialize for ItemInstance<'a> {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_i32(self.network_id)?;
        if self.network_id == 0 {
            // Item is AIR.
            return Ok(())
        }

        writer.write_u16_le(self.count)?;
        writer.write_var_u32(self.metadata)?;
        writer.write_bool(self.stack_id.is_some())?;

        if let Some(stack_id) = self.stack_id {
            writer.write_var_i32(stack_id)?;
        }

        writer.write_var_i32(self.block_runtime_id)?;

        let mut extra = RVec::alloc();
        
        if self.nbt.is_empty() {
            extra.write_i16_le(0)?;
        } else {
            extra.write_i16_le(-1)?; // Length
            extra.write_u8(1)?; // Version
            nbt::to_var_bytes_in(&mut extra, &self.nbt)?;
        }

        extra.write_u32_le(self.can_place_on.len() as u32)?;
        for block in &self.can_place_on {
            extra.write_u16_le(block.len() as u16)?;
            extra.extend_from_slice(block.as_bytes());
        }

        extra.write_u32_le(self.can_destroy.len() as u32)?;
        for block in &self.can_destroy {
            extra.write_u16_le(block.len() as u16)?;
            extra.extend_from_slice(block.as_bytes());
        }

        if self.network_id == SHIELD_ID.load(Ordering::Relaxed) {
            extra.write_i64_le(self.blocking_tick)?;
        }

        writer.write_var_u32(extra.len() as u32)?;
        writer.write_all(&extra)?;

        Ok(())
    }
}

impl<'a> Deserialize<'a> for ItemInstance<'a> {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let network_id = reader.read_var_i32()?;
        // tracing::debug!("Network ID: {network_id}");
        if network_id == 0 {
            // tracing::debug!("Item is air");

            // Item is air
            return Ok(ItemInstance {
                network_id: 0,
                ..Default::default()
            })
        }

        let count = reader.read_u16_le()?;
        // tracing::debug!("Count: {count}");

        let metadata = reader.read_var_u32()?;
        // tracing::debug!("Metadata: {metadata}");

        let has_stack_id = reader.read_bool()?;
        let stack_id = has_stack_id.then(|| reader.read_var_i32()).transpose()?;
        // tracing::debug!("Stack ID: {stack_id:?}");

        let block_runtime_id = reader.read_var_i32()?;
        // tracing::debug!("Block runtime ID: {block_runtime_id}");

        let extra_data_len = reader.read_var_u32()?;
        // let remaining = reader.remaining();

        let mut extra_reader = reader.take_n(extra_data_len as usize)?;


        let length = extra_reader.read_i16_le()?;
        let nbt = if length == -1 {
            let version = extra_reader.read_u8()?;
            if version == 1 {
                let (nbt, n) = nbt::from_var_bytes(&mut extra_reader)?;
                extra_reader.advance(n)?;
                nbt
            } else {
                anyhow::bail!("Invalid item NBT version: {version}");
            }
        } else if length > 0 {
            let (nbt, n) = nbt::from_var_bytes(&mut extra_reader)?;
            extra_reader.advance(n)?;
            nbt
        } else {
            HashMap::new()
        };
        // tracing::debug!("NBT: {nbt:?}");

        let can_place_on_len = extra_reader.read_u32_le()?;
        // tracing::debug!("Can place entries: {can_place_on_len}");
        let mut can_place_on = Vec::with_capacity(can_place_on_len as usize);
        for _ in 0..can_place_on_len {
            let str_len = extra_reader.read_u16_le()?;
            let name = std::str::from_utf8(extra_reader.take_n(str_len as usize)?)?;

            can_place_on.push(name);
        }

        let can_destroy_len = extra_reader.read_u32_le()?;
        // tracing::debug!("Can break entries: {can_destroy_len}");
        let mut can_destroy = Vec::with_capacity(can_destroy_len as usize);
        for _ in 0..can_destroy_len {
            let str_len = extra_reader.read_u16_le()?;
            let name = std::str::from_utf8(extra_reader.take_n(str_len as usize)?)?;

            can_destroy.push(name);
        }

        let blocking_tick = if network_id == SHIELD_ID.load(Ordering::Relaxed) {
            extra_reader.read_i64_le()?
        } else {
            0
        };

        Ok(ItemInstance {
            network_id,
            count,
            metadata,
            stack_id,
            block_runtime_id,
            nbt,
            can_place_on,
            can_destroy,
            blocking_tick
        })
    }
}

#[derive(Debug, Clone)]
pub struct InventoryTransaction<'a> {
    pub legacy_request_id: i32,
    pub legacy_transactions: Vec<LegacyTransactionEntry<'a>>,
    pub transaction_type: TransactionType<'a>,
    pub actions: Vec<TransactionAction<'a>>
}

impl<'a> ConnectedPacket for InventoryTransaction<'a> {
    const ID: u32 = 0x1e;
}

impl<'a> Serialize for InventoryTransaction<'a> {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_i32(self.legacy_request_id)?;
        if self.legacy_request_id != 0 {
            writer.write_var_u32(self.legacy_transactions.len() as u32)?;
            for transaction in &self.legacy_transactions {
                transaction.serialize_into(writer)?;
            }
        }

        writer.write_var_u32(self.transaction_type.as_id())?;

        writer.write_var_u32(self.actions.len() as u32)?;
        for action in &self.actions {
            action.serialize_into(writer)?;
        }

        self.transaction_type.serialize_into(writer)
    }
}

impl<'a> Deserialize<'a> for InventoryTransaction<'a> {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<InventoryTransaction<'a>> {
        let legacy_request_id = reader.read_var_i32()?;
        // tracing::debug!("legacy_request_id: {legacy_request_id}");

        let legacy_transaction_len = if legacy_request_id == 0 { 0 } else { reader.read_var_u32()? };
        let mut legacy_transactions = Vec::with_capacity(legacy_transaction_len as usize);
        for _ in 0..legacy_transaction_len {
            legacy_transactions.push(LegacyTransactionEntry::deserialize_from(reader)?);
        }
        // tracing::debug!("legacy {legacy_transactions:?}");

        let transaction_type = reader.read_var_u32()?;
        
        let actions_len = reader.read_var_u32()?;
        let mut actions = Vec::with_capacity(actions_len as usize);
        for _ in 0..actions_len {
            actions.push(TransactionAction::deserialize_from(reader)?);
        }
        // tracing::debug!("actions {actions:?}");
        
        let transaction_type = TransactionType::deserialize_from(transaction_type, reader)?;
        // tracing::debug!("transaction type2 {transaction_type:?}");

        Ok(InventoryTransaction {
            legacy_request_id,
            legacy_transactions,
            transaction_type,
            actions
        })
    }
}