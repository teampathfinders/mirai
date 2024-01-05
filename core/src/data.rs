use lazy_static::lazy_static;
use level::PaletteEntry;
use nohash_hasher::BuildNoHashHasher;
use std::collections::HashMap;
use tokio_util::bytes::Buf;

lazy_static! {
    pub static ref RUNTIME_ID_DATA: RuntimeIdMap = RuntimeIdMap::new().unwrap();
    pub static ref BLOCK_STATE_DATA: BlockStateMap = BlockStateMap::new().unwrap();
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

        dbg!(&map);

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
        tracing::debug!("Generating block state map...");

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

            if item.name == "minecraft:planks" {
                dbg!(&item);
            }

            if item.name == "minecraft:air" {
                map.air_id = current_id;
            }

            current_id += 1;
        }

        Ok(map)
    }

    pub fn get(&self, block: &PaletteEntry) -> Option<u32> {
        let hash = block.hash();
        let found = self.runtime_hashes.get(&hash).cloned();

        if found.is_none() {
            dbg!(block);
        }

        found
    }
}
