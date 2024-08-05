use std::mem::MaybeUninit;

use proto::bedrock::{InventoryTransaction, WindowId};

#[derive(Debug, Clone)]
pub struct InventorySlot {

}

impl InventorySlot {
    /// Creates an inventory slot that is empty.
    pub const fn empty() -> InventorySlot {
        InventorySlot {}
    }
}

impl Default for InventorySlot {
    #[inline]
    fn default() -> InventorySlot {
        InventorySlot::empty()
    }
}

/// An inventory of generic size `S`. This generic makes it possible to create
/// inventories of different sizes without having to reimplement them.
pub struct Inventory<const S: usize> {
    id: WindowId,
    slots: [InventorySlot; S]
}

impl<const S: usize> Inventory<S> {
    /// Creates an empty inventory.
    pub fn empty(id: WindowId) -> Inventory<S> {
        Inventory { 
            id,
            slots: std::array::from_fn(|_| InventorySlot::empty())
        }
    }

    /// Returns the requested slot in this inventory, panicking if the slot index is out of range.
    /// 
    /// Use [`try_get`](Self::try_get) for a no-panic variant of this function.
    pub const fn get(&self, slot: usize) -> &InventorySlot {
        assert!(slot < S, "Inventory slot out of range");
        &self.slots[slot]
    }

    /// Returns the requested slot in this inventory, returning `None` if the slot index is out of range.
    pub fn try_get(&self, slot: usize) -> Option<&InventorySlot> {
        (slot < S).then(|| &self.slots[slot])
    }

    /// Performs the given transaction to this inventory.
    pub fn transaction(&mut self, transaction: &InventoryTransaction) -> anyhow::Result<()> {
        todo!()
    }
}