use std::io::Write;
use util::Serialize;
use util::bytes::{BinaryWrite, MutableBuffer, VarInt};
use util::Result;

use crate::network::ConnectedPacket;

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

#[derive(Debug, Clone)]
pub struct ItemCollection {
    pub runtime_id: i32,
    pub network_id: u32,
    pub meta: u32,
    /// Amount of items that a single stack holds.
    pub count: u16,
    pub placeable_on: Vec<String>,
    pub can_break: Vec<String>
}

impl ItemCollection {
    pub fn serialized_size(&self) -> usize {
        30
    }

    pub fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_var_i32(self.runtime_id)?;
        if self.network_id == 0 {
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

        if self.network_id == ITEM_ID_SHIELD {
            todo!();
        }

        buffer.write_var_u32(extra_buffer.len() as u32)?;
        buffer.write(&extra_buffer.snapshot())?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CreativeContent<'a> {
    pub items: &'a [ItemCollection],
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
            buffer.write_var_i32(i as i32 + 1)?;
            item.serialize(buffer)?;
        }

        Ok(())
    }
}
