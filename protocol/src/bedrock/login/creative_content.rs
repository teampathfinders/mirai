use std::io::Write;
use util::Serialize;
use util::{BinaryWrite, VarInt};
use util::Result;

use crate::bedrock::ConnectedPacket;

pub const ITEM_ID_SHIELD: u32 = 513;

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
    pub const fn serialized_size(&self) -> usize {
        30
    }

    pub fn serialize<W: BinaryWrite>(&self, network_id: u32, writer: &mut W) -> anyhow::Result<()> {
        if network_id == 0 {
            // Item is air, nothing left to do.
            return Ok(())
        }

        writer.write_u16_le(self.count)?;
        writer.write_var_u32(self.meta)?;
        writer.write_var_i32(self.runtime_id)?;

        let mut extra_writer = Vec::new();

        extra_writer.write_u16_le(0)?;

        extra_writer.write_u32_le(self.placeable_on.len() as u32)?;
        for block in &self.placeable_on {
            extra_writer.write_str(block)?;
        }

        extra_writer.write_u32_le(self.can_break.len() as u32)?;
        for block in &self.can_break {
            extra_writer.write_str(block)?;
        }

        // if self.network_id == ITEM_ID_SHIELD {
        //     todo!();
        // }

        writer.write_var_u32(extra_writer.len() as u32)?;
        writer.write_all(&extra_writer.as_ref())?;

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
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_u32(self.items.len() as u32)?;
        for (i, item) in self.items.iter().enumerate() {
            writer.write_var_u32(i as u32 + 1)?;
            item.serialize(i as u32 + 1, writer)?;
        }

        Ok(())
    }
}
