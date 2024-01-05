use std::io::Write;
use util::Serialize;
use util::{BinaryWrite, MutableBuffer, VarInt};
use util::Result;

use crate::bedrock::ConnectedPacket;

pub const ITEM_ID_SHIELD: u32 = 513;

// /// Represents a combination of a network ID and metadata value.
// #[derive(Debug, Clone)]
// pub struct ItemType {
//     /// Numerical ID of the item.
//     pub network_id: u32,
//     /// Damage value or variant of the item.
//     pub metadata: u32,
// }
//
// /// Represents an item instance/stack.
// #[derive(Debug, Clone)]
// pub struct ItemStack {
//     pub item_type: ItemType,
//     pub runtime_id: u32,
//     pub count: u16,
//     // pub nbt_data: nbt::Value,
//     pub can_be_placed_on: Vec<String>,
//     pub can_break: Vec<String>,
//     pub has_network_id: bool,
// }

/// Represents a stack of items.
#[derive(Debug, Clone)]
pub struct ItemStack {
    /// Runtime ID of the item. This is the ID with which the item is registered in the [`RUNTIME_ID_DATA`](core::RUNTIME_ID_DATA) map.
    pub runtime_id: i32,
    /// Damage value of the item.
    pub meta: u32,
    /// Amount of items that a single stack holds.
    pub count: u16,
    /// On which blocks the item can be placed.
    pub placeable_on: Vec<String>,
    /// Which blocks the item can break.
    pub can_break: Vec<String>
}

impl ItemStack {
    pub fn serialized_size(&self) -> usize {
        30
    }

    pub fn serialize(&self, network_id: u32, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        if network_id == 0 {
            // Item is air, nothing left to do.
            return Ok(())
        }

        buffer.write_u16_le(self.count)?;
        buffer.write_var_u32(self.meta)?;
        buffer.write_var_i32(self.runtime_id)?;

        let mut extra_buffer = MutableBuffer::new();

        extra_buffer.write_u16_le(0)?;

        extra_buffer.write_u32_le(self.placeable_on.len() as u32)?;
        for block in &self.placeable_on {
            extra_buffer.write_str(block)?;
        }

        extra_buffer.write_u32_le(self.can_break.len() as u32)?;
        for block in &self.can_break {
            extra_buffer.write_str(block)?;
        }

        // if self.network_id == ITEM_ID_SHIELD {
        //     todo!();
        // }

        buffer.write_var_u32(extra_buffer.len() as u32)?;
        buffer.write_all(&extra_buffer.snapshot())?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CreativeContent<'a> {
    pub items: &'a [ItemStack],
}

impl ConnectedPacket for CreativeContent<'_> {
    const ID: u32 = 0x91;

    fn serialized_size(&self) -> usize {
        (self.items.len() as u32).var_len() +
            self.items.iter().fold(0, |acc, s| acc + s.serialized_size())
    }
}

impl Serialize for CreativeContent<'_> {
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_var_u32(self.items.len() as u32)?;
        for (i, item) in self.items.iter().enumerate() {
            buffer.write_var_u32(i as u32 + 1)?;
            item.serialize(i as u32 + 1, buffer)?;
        }

        Ok(())
    }
}
