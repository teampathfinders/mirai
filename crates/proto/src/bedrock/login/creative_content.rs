use std::collections::HashMap;

use util::{RString, RVec, Serialize};
use util::{BinaryWrite, VarInt};

use crate::bedrock::ConnectedPacket;

// pub const ITEM_ID_SHIELD: u32 = 513;

// #[derive(Debug, Clone)]
// pub struct ItemStack {
//     /// Runtime ID of the item. This is the ID with which the item is registered in the [`RUNTIME_ID_DATA`](core::RUNTIME_ID_DATA) map.
//     pub runtime_id: i32,
//     /// Damage value of the item.
//     pub meta: u32,
//     /// Amount of items that a single stack holds.
//     pub count: u16,
//     /// On which blocks the item can be placed.
//     pub placeable_on: Vec<String>,
//     /// Which blocks the item can break.
//     pub can_break: Vec<String>
// }

// impl ItemStack {
//     pub const fn serialized_size(&self) -> usize {
//         30
//     }

//     pub fn serialize<W: BinaryWrite>(&self, network_id: u32, writer: &mut W) -> anyhow::Result<()> {
//         if network_id == 0 {
//             // Item is air, nothing left to do.
//             return Ok(())
//         }

//         writer.write_u16_le(self.count)?;
//         writer.write_var_u32(self.meta)?;
//         writer.write_var_i32(self.runtime_id)?;

//         let mut extra_writer = Vec::new();

//         extra_writer.write_u16_le(0)?;

//         extra_writer.write_u32_le(self.placeable_on.len() as u32)?;
//         for block in &self.placeable_on {
//             extra_writer.write_str(block)?;
//         }

//         extra_writer.write_u32_le(self.can_break.len() as u32)?;
//         for block in &self.can_break {
//             extra_writer.write_str(block)?;
//         }

//         // if self.network_id == ITEM_ID_SHIELD {
//         //     todo!();
//         // }

//         writer.write_var_u32(extra_writer.len() as u32)?;
//         writer.write_all(extra_writer.as_ref())?;

//         Ok(())
//     }
// }

#[derive(Debug, Clone)]
pub struct ItemType {
    pub network_id: i32,
    pub meta: u32
}

#[derive(Debug, Clone)]
pub struct ItemStack {
    pub item_type: ItemType,
    pub block_runtime_id: i32,
    pub count: u16,
    pub nbt_data: HashMap<RString, nbt::Value>,
    pub can_be_placed_on: Vec<RString>,
    pub can_break: Vec<RString>,
    pub has_network_id: bool
}

impl Serialize for ItemStack {
    fn size_hint(&self) -> Option<usize> {
        None
    }

    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_i32(self.item_type.network_id)?;
        if self.item_type.network_id == 0 {
            // Item is air, no more data
            return Ok(())
        }

        writer.write_u16_le(self.count)?;
        writer.write_var_u32(self.item_type.meta)?;
        writer.write_var_i32(self.block_runtime_id)?;
        
        let extra_data_size = 0;
        let mut extra_data = RVec::alloc_with_capacity(extra_data_size);
            
        if self.nbt_data.is_empty() {
            extra_data.write_i16_le(0)?; // Length = 0
        } else {
            extra_data.write_i16_le(-1)?; // Length
            extra_data.write_u8(1)?; // Version
            nbt::to_var_bytes_in(&mut extra_data, &self.nbt_data)?;
        }

        writer.write_u32_le(self.can_be_placed_on.len() as u32)?;
        for block in &self.can_be_placed_on {
            writer.write_str(block)?;
        }

        writer.write_u32_le(self.can_break.len() as u32)?;
        for block in &self.can_break {
            writer.write_str(block)?;
        }

        const SHIELD_ID: i32 = 355;

        if self.item_type.network_id == SHIELD_ID {
            extra_data.write_i64_le(0)?; // Blocking tick
        }

        writer.write_all(&extra_data)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CreativeItem {
    pub creative_network_id: u32,
    pub item: ItemStack
}

impl Serialize for CreativeItem {
    fn size_hint(&self) -> Option<usize> {
        let size = self.creative_network_id.var_len() + self.item.size_hint().unwrap_or(0);
        Some(size)
    }

    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_u32(self.creative_network_id)?;
        self.item.serialize_into(writer)
    }
}

#[derive(Debug, Clone)]
pub struct CreativeContent<'a> {
    pub items: &'a [CreativeItem],
}

impl ConnectedPacket for CreativeContent<'_> {
    const ID: u32 = 0x91;

    fn serialized_size(&self) -> usize {
        (self.items.len() as u32).var_len() +
            self.items.iter().fold(0, |acc, s| acc + s.size_hint().unwrap())
    }
}

impl Serialize for CreativeContent<'_> {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_u32(self.items.len() as u32)?;
        for item in self.items {
            item.serialize_into(writer)?;
        }

        Ok(())
    }
}
