use std::collections::HashMap;
use std::io::Write;
use util::bytes::{BinaryWrite, MutableBuffer};
use util::Serialize;

const NETWORK_ID_AIR: u32 = 0;
const NETWORK_ID_SHIELD: u32 = 0xff;

#[derive(Debug, Clone)]
pub struct ItemType {
    /// ID of the item.
    pub network_id: u32,
    /// Damage value or variant of the item.
    pub meta: u32
}

#[derive(Debug, Clone)]
pub struct ItemStack {
    pub ty: ItemType,
    pub block_id: u32,
    /// Amount of items that the stack holds.
    pub count: u16,
    pub properties: HashMap<String, nbt::Value>,
    pub can_be_placed_on: Vec<String>,
    pub can_break: Vec<String>,
    pub has_network_id: bool
}

impl Serialize for ItemStack {
    fn serialize<W>(&self, buffer: W) -> anyhow::Result<()> where W: BinaryWrite {
        buffer.write_var_u32(self.ty.network_id)?;
        if self.ty.network_id == NETWORK_ID_AIR {
            return Ok(())
        }

        buffer.write_u16_le(self.count)?;
        buffer.write_var_u32(self.ty.meta)?;
        buffer.write_var_u32(self.block_id)?;

        let mut extra = MutableBuffer::new();

        if self.properties.is_empty() {
            extra.write_i16_le(0)?;
        } else {
            extra.write_i16_le(-1)?;
            extra.write_u8(1);
            nbt::to_var_bytes_in(&mut extra, &self.properties)?;
        }

        extra.write_u32_le(self.can_be_placed_on.len() as u32)?;
        for block in &self.can_be_placed_on {
            extra.write_u16_le(block.len() as u16)?;
            extra.write_all(block.as_bytes())?;
        }

        extra.write_u32_le(self.can_break.len() as u32)?;
        for block in &self.can_break {
            extra.write_u16_le(block.len() as u16)?;
            extra.write_all(block.as_bytes())?;
        }

        if self.ty.network_id == NETWORK_ID_SHIELD {
            extra.write_i64_le(0)?; // BLocking tick.
        }

        buffer.write_var_u32(extra.len() as u32)?;
        buffer.write_all(&extra.snapshot())?;

        Ok(())
    }
}