use std::collections::HashMap;
use util::Serialize;
use util::bytes::{BinaryWrite, MutableBuffer, VarInt};
use util::Result;

use crate::network::ConnectedPacket;

pub const ITEM_ID_SHIELD: u32 = 513;

/// Represents a combination of a network ID and metadata value.
#[derive(Debug, Clone)]
pub struct ItemType {
    /// Numerical ID of the item.
    pub network_id: u32,
    /// Damage value or variant of the item.
    pub metadata: u32,
}

/// Represents an item instance/stack.
#[derive(Debug, Clone)]
pub struct ItemStack {
    pub item_type: ItemType,
    pub runtime_id: u32,
    pub count: u16,
    // pub nbt_data: nbt::Value,
    pub properties: HashMap<String, nbt::Value>,
    pub can_be_placed_on: Vec<String>,
    pub can_break: Vec<String>,
    pub has_network_id: bool,
}

impl ItemStack {
    pub fn serialized_size(&self) -> usize {
        0
        // todo!();
        // self.item_type.network_id.var_len() +
        // if self.item_type.network_id != 0 {
        //     2 +
        //     self.item_type.metadata.var_len() +
        //     self.runtime_id.var_len() +
        //     2 +
        //     if let Value::Compound(ref map) = self.nbt_data {
        //         if !map.is_empty() {
        //             1 + self.nbt_data.serialized_net_size("")
        //         } else {
        //             0
        //         }
        //     } else {
        //         0
        //     }
        // } else {
        //     0
        // }
    }

    pub fn serialize(&self, mut buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_var_u32(self.item_type.network_id)?;
        if self.item_type.network_id == 0 {
            // Air has no data.
            return Ok(());
        }

        buffer.write_u16_be(self.count)?;
        buffer.write_var_u32(self.item_type.metadata)?;
        buffer.write_var_u32(self.runtime_id)?;

        if self.properties.is_empty() {
            buffer.write_i16_le(0); // Length
        } else {
            buffer.write_i16_le(-1);
            buffer.write_u8(1);

            nbt::to_var_bytes_in(&mut buffer, &self.properties)?;
        }

        buffer.write_u32_be(self.can_be_placed_on.len() as u32)?;
        for item in &self.can_be_placed_on {
            buffer.write_str(item)?;
        }

        buffer.write_u32_be(self.can_break.len() as u32)?;
        for item in &self.can_break {
            buffer.write_str(item)?;
        }

        if self.item_type.network_id == ITEM_ID_SHIELD {
            buffer.write_u64_be(0)?; // Blocking tick.
        }

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
        for item in self.items {
            item.serialize(buffer)?;
        }

        Ok(())
    }
}
