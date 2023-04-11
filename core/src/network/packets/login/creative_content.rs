use std::collections::HashMap;
use util::Serialize;
use util::bytes::{BinaryWrite, MutableBuffer, VarInt};
use util::Result;
use crate::item::ItemStack;

use crate::network::ConnectedPacket;

/// An item available in the creative menu.
#[derive(Debug, Clone)]
pub struct CreativeItem {
    pub network_id: u32,
    pub stack: ItemStack
}

impl Serialize for CreativeItem {
    fn serialize<W>(&self, buffer: W) -> anyhow::Result<()> where W: BinaryWrite {
        buffer.write_var_u32(self.network_id)?;
        self.stack.serialize(buffer)
    }
}

/// Gives a list of items available in the creative inventory.
#[derive(Debug, Clone)]
pub struct CreativeContent<'a> {
    /// Available items.
    pub items: &'a [CreativeItem],
}

impl<'a> ConnectedPacket for CreativeContent<'a> {
    const ID: u32 = 0x91;

    // TODO: Implement properly.
    fn serialized_size(&self) -> usize {
        // (self.items.len() as u32).var_len()
        0
    }
}

impl<'a> Serialize for CreativeContent<'a> {
    fn serialize<W>(&self, buffer: W) -> anyhow::Result<()> where W: BinaryWrite {
        buffer.write_var_u32(self.items.len() as u32)?;
        for item in self.items {
            item.serialize(buffer)?;
        }

        Ok(())
    }
}
