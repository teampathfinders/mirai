use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref RUNTIME_ID_MAP: RuntimeIdMap = RuntimeIdMap::new().unwrap();
}

#[derive(Debug)]
pub struct RuntimeIdMap {
    map: HashMap<String, i32>
}

impl RuntimeIdMap {
    pub fn new() -> anyhow::Result<Self> {
        tracing::debug!("Generating item runtime ID map");

        const BYTES: &[u8] = include_bytes!("../include/item_runtime_ids.nbt");
        let map = nbt::from_var_bytes(BYTES)?.0;

        Ok(Self { map })
    }

    pub fn get(&self, name: &str) -> Option<i32> {
        todo!()
    }
}