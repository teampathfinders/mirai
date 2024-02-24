use util::{BinaryRead, BlockPosition, Deserialize, Vector};

use crate::bedrock::ConnectedPacket;

use super::WindowId;

#[derive(Debug, Clone)]
pub struct LegacyTransactionEntry<'a> {
    pub container_id: u8,
    pub changed_slots: &'a [u8]
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

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Item {

}

#[derive(Debug, Clone)]
pub struct TransactionAction {
    pub source_type: TransactionSourceType,
    pub slot: u32,
    pub old_item: Item,
    pub new_item: Item
}

impl<'a> Deserialize<'a> for TransactionAction {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let source_type = TransactionSourceType::deserialize_from(reader)?;
        let slot = reader.read_var_u32()?;

        Ok(TransactionAction {
            source_type,
            slot,
            old_item: Item {},
            new_item: Item {}
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
pub enum TransactionType {
    #[default]
    Normal,
    Mismatch,
    Use {
        action_type: UseItemAction,
        block_position: BlockPosition,
        face: i32,
        hotbar_slot: i32,
        held_item: Item,
        player_position: Vector<f32, 3>,
        click_position: Vector<f32, 3>,
        block_runtime_id: u32
    },
    UseOnEntity {
        entity_runtime_id: u64,
        action_type: UseOnEntityAction,
        hotbar_slot: i32,
        held_item: Item,
        player_position: Vector<f32, 3>,
        click_position: Vector<f32, 3>
    },
    Release {
        action_type: ReleaseAction,
        hotbar_slot: i32,
        held_item: Item,
        head_position: Vector<f32, 3>
    }
}

impl TransactionType {
    pub fn deserialize_from<'a, R: BinaryRead<'a>>(transaction_type: u32, reader: &mut R) -> anyhow::Result<TransactionType> {
        Ok(match transaction_type {
            0 => Self::Normal,
            1 => Self::Mismatch,
            2 => Self::Use {
                action_type: UseItemAction::try_from(reader.read_var_u32()?)?,
                block_position: reader.read_block_pos()?,
                face: reader.read_i32_le()?,
                hotbar_slot: reader.read_i32_le()?,
                held_item: Item::deserialize_from(reader)?,
                player_position: reader.read_vecf()?,
                click_position: reader.read_vecf()?,
                block_runtime_id: reader.read_var_u32()?
            },
            3 => Self::UseOnEntity {
                entity_runtime_id: reader.read_var_u64()?,
                action_type: UseOnEntityAction::try_from(reader.read_var_u32()?)?,
                hotbar_slot: reader.read_i32_le()?,
                held_item: Item::deserialize_from(reader)?,
                player_position: reader.read_vecf()?,
                click_position: reader.read_vecf()?
            },
            4 => Self::Release {
                action_type: ReleaseAction::try_from(reader.read_var_u32()?)?,
                hotbar_slot: reader.read_i32_le()?,
                held_item: Item::deserialize_from(reader)?,
                head_position: reader.read_vecf()?
            },
            _ => anyhow::bail!("Invalid transaction type")
        })
    }
}

#[derive(Debug, Clone)]
pub struct InventoryTransaction<'a> {
    pub legacy_request_id: i32,
    pub legacy_transactions: Vec<LegacyTransactionEntry<'a>>,
    pub transaction_type: TransactionType,
    pub actions: Vec<TransactionAction>
}

impl<'a> ConnectedPacket for InventoryTransaction<'a> {
    const ID: u32 = 0x1e;
}

impl<'a> Deserialize<'a> for InventoryTransaction<'a> {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<InventoryTransaction<'a>> {
        let legacy_request_id = reader.read_var_i32()?;
        let legacy_transaction_len = if legacy_request_id == 0 {
            0
        } else {
            reader.read_var_u32()? 
        };

        let mut legacy_transactions = Vec::with_capacity(legacy_transaction_len as usize);
        for _ in 0..legacy_transaction_len {
            legacy_transactions.push(LegacyTransactionEntry::deserialize_from(reader)?);
        }

        let transaction_type = reader.read_var_u32()?;
        let actions_len = reader.read_var_u32()?;

        let mut actions = Vec::with_capacity(actions_len as usize);
        for _ in 0..actions_len {
            actions.push(TransactionAction::deserialize_from(reader)?);
        }

        let transaction_type = TransactionType::deserialize_from(transaction_type, reader)?;

        Ok(InventoryTransaction {
            legacy_request_id,
            legacy_transactions,
            transaction_type,
            actions
        })
    }
}