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

pub struct ItemRegistry {

}

impl ItemRegistry {
    /// Loads the creative items from `creative_items.nbt`.
    pub fn new() -> anyhow::Result<Self> {
        let nbt_bytes: &[u8] = include_bytes!("../../included/creative_items.nbt");
        let creative_nbt: Vec<CreativeItemEntry> = nbt::from_var_bytes(nbt_bytes)?.0;

        dbg!(creative_nbt);

        Ok(Self {})
    }

    pub fn get_creative_items(&self) -> &[CreativeItem] {
        todo!();
    }
}