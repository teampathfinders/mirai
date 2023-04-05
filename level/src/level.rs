// Special keys

use crate::database::Database;
use crate::level_dat::LevelSettings;
use crate::{Dimension, SubChunk};
use anyhow::anyhow;
use lru::LruCache;
use std::fs::File;
use std::io::Read;
use std::num::NonZeroUsize;
use std::path::Path;
use util::Vector;

// Option::unwrap is not const stable yet.
const SUB_CHUNK_CACHE_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(25) };

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SubChunkCacheKey {
    xz: Vector<i32, 2>,
    y: i8,
    dimension: Dimension,
}

pub struct DataProvider {
    database: Database,
    sub_chunks: LruCache<SubChunkCacheKey, SubChunk>,
}

impl DataProvider {
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        let database = Database::open(path.join("db").to_str().ok_or_else(|| anyhow!("Invalid level path"))?)?;

        Ok(Self {
            database,
            sub_chunks: LruCache::new(SUB_CHUNK_CACHE_SIZE),
        })
    }
}

pub struct Level {
    settings: LevelSettings,
    provider: DataProvider,
}

impl Level {
    pub fn open<P>(path: P) -> anyhow::Result<Level>
    where
        P: AsRef<Path>,
    {
        let mut settings_file = File::open(path.as_ref().join("level.dat"))?;
        let mut settings_bin = Vec::new();
        settings_file.read_to_end(&mut settings_bin)?;

        let settings: LevelSettings = nbt::from_le_bytes(&settings_bin[8..])?.0;
        let provider = DataProvider::new(path.as_ref())?;

        Ok(Level { settings, provider })
    }

    pub fn flush(&self) -> anyhow::Result<()> {
        todo!();
    }
}
