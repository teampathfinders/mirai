use util::{BinaryRead, Deserialize, Serialize};

use crate::bedrock::ConnectedPacket;

use super::{ItemInstance, WindowId};

/// Sent when an entity when it changes the item is holding.
/// This is also sent for players when they scroll through their hotbar.
#[derive(Debug, Clone)]
pub struct MobEquipment<'a> {
    /// Runtime ID of the entity.
    pub runtime_id: u64,
    /// Item the entity will now be holding.
    pub new_item: ItemInstance<'a>,
    /// Hotbar slot 
    pub hotbar_slot: u8,
    /// Window that had its equipped hand changed. This is used to differentiate
    /// between inventories such as off-hand and main hand.
    pub window_id: WindowId
}

impl<'a> ConnectedPacket for MobEquipment<'a> {
    const ID: u32 = 0x1f;
}

impl<'a> Deserialize<'a> for MobEquipment<'a> {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let runtime_id = reader.read_var_u64()?;
        let new_item = ItemInstance::deserialize_from(reader)?;
        let _slot = reader.read_u8()?; // InventorySlot, only for backwards compatibility and always equal to HotbarSlot.
        let hotbar_slot = reader.read_u8()?;
        let window_id = WindowId::try_from(reader.read_u8()? as i32)?;

        Ok(MobEquipment {
            runtime_id, new_item, hotbar_slot, window_id
        })
    }
}

impl<'a> Serialize for MobEquipment<'a> {
    fn serialize_into<W: util::BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_u64(self.runtime_id)?;
        self.new_item.serialize_into(writer)?;
        writer.write_u8(self.hotbar_slot)?; // InventorySlot, only for backwards compatibility and always equal to HotbarSlot.
        writer.write_u8(self.hotbar_slot)?;
        writer.write_u8(Into::<i32>::into(self.window_id) as u8)
    }
}