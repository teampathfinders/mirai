use std::collections::HashMap;

use bytes::{BufMut, BytesMut};
use nbt::Value;

use common::VResult;
use crate::network::Encodable;
use crate::network::packets::GamePacket;
use common::WriteExtensions;

pub const ITEM_ID_SHIELD: u32 = 513;

/// Represents a combination of a network ID and metadata value.
#[derive(Debug)]
pub struct ItemType {
    /// Numerical ID of the item.
    pub network_id: u32,
    /// Damage value or variant of the item.
    pub metadata: u32,
}

/// Represents an item instance/stack.
#[derive(Debug)]
pub struct ItemStack {
    pub item_type: ItemType,
    pub runtime_id: u32,
    pub count: u16,
    pub nbt_data: nbt::Value,
    pub can_be_placed_on: Vec<String>,
    pub can_break: Vec<String>,
    pub has_network_id: bool,
}

impl ItemStack {
    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_var_u32(self.item_type.network_id);
        if self.item_type.network_id == 0 {
            // Air has no data.
            return;
        }

        buffer.put_u16(self.count);
        buffer.put_var_u32(self.item_type.metadata);
        buffer.put_var_u32(self.runtime_id);

        if let Value::Compound(ref map) = self.nbt_data {
            let length = map.len();
            if length == 0 {
                buffer.put_i16(0); // Length
            } else {
                buffer.put_i16(-1); // Length
                buffer.put_u8(1); // Version
                nbt::RefTag {
                    name: "",
                    value: &self.nbt_data,
                }.encode_with_le(buffer);
            }
        } else {
            todo!()
        }

        buffer.put_u32(self.can_be_placed_on.len() as u32);
        for item in &self.can_be_placed_on {
            buffer.put_string(item);
        }

        buffer.put_u32(self.can_break.len() as u32);
        for item in &self.can_break {
            buffer.put_string(item);
        }

        if self.item_type.network_id == ITEM_ID_SHIELD {
            buffer.put_u64(0); // Blocking tick.
        }
    }
}

#[derive(Debug)]
pub struct CreativeItem {
    pub network_id: u32,
    pub stack: ItemStack,
}

impl CreativeItem {
    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_var_u32(self.network_id);
        self.stack.encode(buffer);
    }
}

#[derive(Debug)]
pub struct CreativeContent {
    pub items: Vec<CreativeItem>,
}

impl GamePacket for CreativeContent {
    const ID: u32 = 0x91;
}

impl Encodable for CreativeContent {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_var_u32(self.items.len() as u32);
        for item in &self.items {
            item.encode(&mut buffer);
        }

        Ok(buffer)
    }
}