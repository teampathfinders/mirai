use lazy_static::lazy_static;
use level::PaletteEntry;
use nohash_hasher::BuildNoHashHasher;
use proto::bedrock::ItemStack;
use std::collections::HashMap;
use serde::Deserialize;
use tokio_util::bytes::Buf;

lazy_static! {
    pub static ref RUNTIME_ID_DATA: RuntimeIdMap = RuntimeIdMap::new().unwrap();
    pub static ref BLOCK_STATE_DATA: BlockStateMap = BlockStateMap::new().unwrap();
    pub static ref CREATIVE_ITEMS_DATA: CreativeItemsMap = CreativeItemsMap::new().unwrap();
}

#[derive(Debug)]
pub struct RuntimeIdMap {
    map: HashMap<String, i32>,
}

impl RuntimeIdMap {
    pub fn new() -> anyhow::Result<Self> {
        tracing::debug!("Generating item runtime ID map...");

        const BYTES: &[u8] = include_bytes!("../include/item_runtime_ids.nbt");
        let map = nbt::from_var_bytes(BYTES)?.0;

        Ok(Self { map })
    }

    pub fn get(&self, name: &str) -> Option<i32> {
        self.map.get(name).cloned()
    }
}

#[derive(Debug, Default)]
pub struct BlockStateMap {
    /// Converts state hashes to runtime IDs.
    runtime_hashes: HashMap<u64, u32, BuildNoHashHasher<u64>>,
    air_id: u32,
}

impl BlockStateMap {
    pub fn new() -> anyhow::Result<Self> {
        tracing::debug!("Generating block state data...");

        const BYTES: &[u8] = include_bytes!("../include/block_states.nbt");
        const STATE_COUNT: usize = 14127;
        let mut reader = BYTES;

        let mut map = BlockStateMap::default();
        map.runtime_hashes.reserve(STATE_COUNT);

        let mut current_id = 0;
        while reader.has_remaining() {
            let (item, n): (PaletteEntry, usize) = nbt::from_var_bytes(reader).unwrap();
            reader = reader.split_at(n).1;

            let state_hash = item.hash();
            map.runtime_hashes.insert(state_hash, current_id);

            if item.name == "minecraft:air" {
                map.air_id = current_id;
            }

            current_id += 1;
        }

        assert_eq!(STATE_COUNT, current_id as usize);

        Ok(map)
    }

    pub fn get(&self, block: &PaletteEntry) -> Option<u32> {
        let hash = block.hash();
        let found = self.runtime_hashes.get(&hash).cloned();

        // if found.is_none() {
        //     dbg!(block);
        // }

        found
    }
}

#[derive(Debug, Deserialize)]
pub struct CreativeItemsEntry {
    /// Name of the creative item.
    pub name: String,
    pub meta: i16,
    /// This field only exists if the given item has NBT data. This can be a command block or chest with data for example.
    pub nbt: Option<HashMap<String, nbt::Value>>,
    /// This field only exists if the given item is a block.
    pub block_properties: Option<HashMap<String, nbt::Value>>
}

#[derive(Debug)]
pub struct CreativeItemsMap {
    pub item_stacks: Vec<ItemStack>
}

impl CreativeItemsMap {
    pub fn new() -> anyhow::Result<Self> {
        tracing::debug!("Generating creative items data...");

        const BYTES: &[u8] = include_bytes!("../include/creative_items.nbt");
        let items: Vec<CreativeItemsEntry> = nbt::from_var_bytes(BYTES)?.0;

        let mut item_stacks = Vec::with_capacity(items.len());
        for item in &items[..10] {
            let runtime_id = if let Some(rid) = RUNTIME_ID_DATA.get(&item.name) {
                rid
            } else {
                continue
            };

            let stack = if let Some(_properties) = &item.block_properties {
                ItemStack {
                    runtime_id,
                    meta: item.meta as u32,
                    count: 64,
                    can_break: vec![],
                    placeable_on: vec![]
                }
            } else {
                ItemStack {
                    runtime_id,
                    meta: item.meta as u32,
                    count: 1,
                    can_break: vec![],
                    placeable_on: vec![]
                }
            };

            item_stacks.push(stack);
        }

        Ok(Self { item_stacks })
    }

    pub fn items(&self) -> &[ItemStack] {
        &self.item_stacks
    }
}