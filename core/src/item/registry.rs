use std::collections::HashMap;
use crate::network::CreativeItem;

#[derive(serde::Deserialize, Debug)]
pub struct CreativeBlock {
    pub name: String,
    pub properties: Option<HashMap<String, nbt::Value>>,
    pub version: i32
}

#[derive(serde::Deserialize, Debug)]
pub struct CreativeItemEntry {
    pub name: String,
    pub meta: i16,
    pub nbt: HashMap<String, nbt::Value>,
    pub block: Option<CreativeBlock>
}

#[derive(Debug)]
pub struct ItemRegistry {
    creative: Vec<CreativeItemEntry>
}

impl ItemRegistry {
    /// Loads the creative items from `creative_items.nbt`.
    pub fn new() -> anyhow::Result<Self> {
        let nbt_data: &[u8] = include_bytes!("../../included/creative_items.nbt");
        let creative: Vec<CreativeItemEntry> = nbt::from_var_bytes(nbt_data)?.0;

        Ok(Self {
            creative
        })
    }

    /// Returns a list of items available in the creative menu.
    #[inline]
    pub fn get_creative_items(&self) -> &[CreativeItemEntry] {
        &self.creative
    }
}