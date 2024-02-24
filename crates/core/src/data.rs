// //! Contains data used throughout the server.

// use level::PaletteEntry;
// use nohash_hasher::BuildNoHashHasher;
// use proto::bedrock::ItemStack;
// use serde::Deserialize;
// use std::collections::HashMap;
// use tokio_util::bytes::Buf;

// /// Maps items to runtime IDs.
// #[derive(Debug)]
// pub struct RuntimeIdMap {
//     map: HashMap<String, i32>,
// }

// impl RuntimeIdMap {
//     /// Creates a new runtime ID map.
//     pub fn new() -> anyhow::Result<Self> {
//         tracing::debug!("Generating item runtime ID map...");

//         const BYTES: &[u8] = include_bytes!("../include/item_runtime_ids.nbt");
//         let map = nbt::from_var_bytes(BYTES)?.0;

//         Ok(Self { map })
//     }

//     /// Gets the runtime ID of an item.
//     pub fn get(&self, name: &str) -> Option<i32> {
//         self.map.get(name).copied()
//     }
// }

// /// Maps block states to runtime IDs.
// #[derive(Debug, Default)]
// pub struct BlockStateMap {
//     /// Converts state hashes to runtime IDs.
//     runtime_hashes: HashMap<u64, u32, BuildNoHashHasher<u64>>,
//     air_id: u32,
// }

// impl BlockStateMap {
//     /// Creates a new block state map.
//     ///
//     /// # Panics
//     ///
//     /// This function panics if the deserialized state count is not equal to the expected count.
//     pub fn new() -> anyhow::Result<Self> {
//         tracing::debug!("Generating block state data...");

//         const BYTES: &[u8] = include_bytes!("../include/block_states.nbt");
//         const STATE_COUNT: usize = 14127;
//         let mut reader = BYTES;

//         let mut map = BlockStateMap::default();
//         map.runtime_hashes.reserve(STATE_COUNT);

//         let mut current_id = 0;
//         while reader.has_remaining() {
//             let (item, n): (PaletteEntry, usize) = nbt::from_var_bytes(reader)?;
//             reader = reader.split_at(n).1;

//             let state_hash = item.hash();
//             map.runtime_hashes.insert(state_hash, current_id);

//             if item.name == "minecraft:air" {
//                 map.air_id = current_id;
//             }

//             current_id += 1;
//         }

//         assert_eq!(STATE_COUNT, current_id as usize, "Missing block state");

//         Ok(map)
//     }

//     /// Gets the runtime ID of a block state.
//     pub fn get(&self, block: &PaletteEntry) -> Option<u32> {
//         let hash = block.hash();
//         let found = self.runtime_hashes.get(&hash).cloned();

//         // if found.is_none() {
//         //     dbg!(block);
//         // }

//         found
//     }
// }

// /// Entry in the [`CreativeItemsMap`].
// #[derive(Debug, Deserialize)]
// pub struct CreativeItemsEntry {
//     /// Name of the creative item.
//     pub name: String,
//     /// Metadata value of the item.
//     pub meta: i16,
//     /// This field only exists if the given item has NBT data. This can be a command block or chest with data for example.
//     pub nbt: Option<HashMap<String, nbt::Value>>,
//     /// This field only exists if the given item is a block.
//     pub block_properties: Option<HashMap<String, nbt::Value>>,
// }

// /// List of items that are available in creative mode.
// #[derive(Debug)]
// pub struct CreativeItemsMap {
//     /// Creative items available on the server.
//     pub item_stacks: Vec<ItemStack>,
// }

// impl CreativeItemsMap {
//     /// Creates a new item map.
//     pub fn new() -> anyhow::Result<Self> {
//         tracing::debug!("Generating creative items data...");

//         const BYTES: &[u8] = include_bytes!("../include/creative_items.nbt");
//         let items: Vec<CreativeItemsEntry> = nbt::from_var_bytes(BYTES)?.0;

//         let item_stacks = Vec::with_capacity(items.len());
//         for _item in &items[..10] {
//             todo!();
//             // let runtime_id = if let Some(rid) = RUNTIME_ID_DATA.get(&item.name) {
//             //     rid
//             // } else {
//             //     continue;
//             // };

//             // let stack = if let Some(_properties) = &item.block_properties {
//             //     ItemStack {
//             //         runtime_id,
//             //         meta: item.meta as u32,
//             //         count: 64,
//             //         can_break: vec![],
//             //         placeable_on: vec![],
//             //     }
//             // } else {
//             //     ItemStack {
//             //         runtime_id,
//             //         meta: item.meta as u32,
//             //         count: 1,
//             //         can_break: vec![],
//             //         placeable_on: vec![],
//             //     }
//             // };

//             // item_stacks.push(stack);
//         }

//         Ok(Self { item_stacks })
//     }

//     /// Returns the items contained in the map.
//     pub fn items(&self) -> &[ItemStack] {
//         &self.item_stacks
//     }
// }

use std::collections::HashMap;

use level::PaletteEntry;
use nohash_hasher::{BuildNoHashHasher, IntMap};
use proto::bedrock::{ItemStack, ItemType};
use util::{BinaryRead, RString};

const CREATIVE_ITEMS_RAW: &[u8] = include_bytes!("../include/creative_items.nbt");

#[derive(Debug, serde::Deserialize)]
struct RawCreativeItem {
    pub name: String,
    pub meta: i16,
    #[serde(default)]
    pub nbt: HashMap<String, nbt::Value>,
    #[serde(default)]
    pub block_properties: HashMap<String, nbt::Value>,
}

pub struct CreativeItems {
    pub(crate) stacks: Vec<ItemStack>,
}

impl CreativeItems {
    pub fn new(item_ids: &ItemNetworkIds, block_states: &BlockStates) -> anyhow::Result<Self> {
        tracing::debug!("Loading creative items");

        let nbt: Vec<RawCreativeItem> = nbt::from_var_bytes(CREATIVE_ITEMS_RAW)?.0;
        let mut stacks = Vec::with_capacity(nbt.len());

        stacks.push(ItemStack {
            item_type: ItemType { network_id: 0, meta: 0 },
            can_destroy: vec![],
            can_place_on: vec![],
            count: 0,
            block_runtime_id: 0,
            nbt_data: HashMap::new(),
        });

        for item in nbt.into_iter().take(20) {
            if item.block_properties.is_empty() {
                let Some(runtime_id) = item_ids.get_id(&item.name) else { continue };

                let stack = ItemStack {
                    item_type: ItemType {
                        network_id: runtime_id,
                        meta: item.meta as u32,
                    },
                    block_runtime_id: 0,
                    count: 1,
                    nbt_data: item.nbt,
                    can_place_on: vec![],
                    can_destroy: vec![],
                };

                stacks.push(stack);
            } else {
                let Some(runtime_id) = block_states.get(&item) else { continue };

                println!("runtime_id: {runtime_id}");

                let stack = ItemStack {
                    item_type: ItemType {
                        network_id: runtime_id as i32,
                        meta: item.meta as u32,
                    },
                    block_runtime_id: runtime_id as i32,
                    count: 1,
                    nbt_data: item.nbt,
                    can_place_on: vec![],
                    can_destroy: vec![],
                };

                stacks.push(stack);
            }
        }

        Ok(Self { stacks })
    }
}

const ITEM_IDS_RAW: &[u8] = include_bytes!("../include/item_runtime_ids.nbt");

/// Mapping between item names and IDs.
#[derive(Debug, Default)]
pub struct ItemNetworkIds {
    /// Converts item names to their network IDs.
    name_to_id: HashMap<String, i32>,
    /// Converts network IDs to item names.
    id_to_name: IntMap<i32, String>,
    /// The network ID of a shield.
    /// Shields get special treatment in ItemStack, so this needs to be known.
    shield_id: i32,
}

impl ItemNetworkIds {
    /// Creates a new item map.
    pub fn new() -> anyhow::Result<Self> {
        tracing::debug!("Loading item identifiers");

        let nbt: HashMap<String, i32> = nbt::from_var_bytes(ITEM_IDS_RAW)?.0;
        let mut shield_id = i32::MAX;

        let mut name_to_id = HashMap::with_capacity(nbt.len());
        for (name, id) in &nbt {
            if name == "minecraft:shield" {
                shield_id = *id;
            }

            name_to_id.insert(name.clone(), *id);
        }

        if shield_id == i32::MAX {
            anyhow::bail!("Unable to find shield network ID");
        }

        let mut id_to_name = IntMap::with_capacity_and_hasher(nbt.len(), BuildNoHashHasher::default());
        for (name, id) in nbt {
            id_to_name.insert(id, name);
        }

        Ok(Self { name_to_id, id_to_name, shield_id })
    }

    /// Convert an item name to a network ID.
    #[inline]
    pub fn get_id(&self, name: &str) -> Option<i32> {
        self.name_to_id.get(name).copied()
    }

    /// Convert an item network ID to a name.
    #[inline]
    pub fn get_name(&self, id: i32) -> Option<&str> {
        self.id_to_name.get(&id).map(|x| x.as_str())
    }
}

const BLOCK_STATES_RAW: &[u8] = include_bytes!("../include/block_states.nbt");

/// Maps block states to runtime IDs.
#[derive(Debug, Default)]
pub struct BlockStates {
    /// Converts state hashes to runtime IDs.
    runtime_hashes: HashMap<u64, u32, BuildNoHashHasher<u64>>,
    air_id: u32,
}

impl BlockStates {
    /// Creates a new block state map.
    ///
    /// # Panics
    ///
    /// This function panics if the deserialized state count is not equal to the expected count.
    pub fn new() -> anyhow::Result<Self> {
        tracing::debug!("Loading block state data");

        const STATE_COUNT: usize = 14127;
        let mut reader = BLOCK_STATES_RAW;

        let mut states = Self {
            runtime_hashes: HashMap::with_capacity_and_hasher(STATE_COUNT, BuildNoHashHasher::default()),
            air_id: 0,
        };

        while reader.remaining() > 0 {
            let (item, n) = nbt::from_var_bytes(reader)?;
            states.register(item)?;

            (_, reader) = reader.split_at(n);
        }

        Ok(states)
    }

    pub fn get(&self, item: &RawCreativeItem) -> Option<u32> {
        let state = PaletteEntry {
            name: item.name.clone(),
            states: item.nbt.clone(),
            version: None,
        };

        if state.name.contains("grass") {
            println!("get {state:?}")
        }

        let hash = state.hash();
        self.runtime_hashes.get(&hash).copied()
    }

    pub fn register(&mut self, state: PaletteEntry) -> anyhow::Result<()> {
        let hash = state.hash();
        let new_id = self.runtime_hashes.len() + 1;
        if state.name == "minecraft:air" {
            self.air_id = new_id as u32;
        }

        if state.name.contains("grass") {
            println!("register {state:?}")
        }

        self.runtime_hashes.insert(hash, new_id as u32);

        Ok(())
    }
}
