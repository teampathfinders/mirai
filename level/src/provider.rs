// Special keys

use crate::biome::ChunkBiome;
use crate::database::Database;
use crate::{DataKey, Dimension, KeyType};
use anyhow::anyhow;
use std::path::Path;
use util::Vector;

pub struct Provider {
    database: Database,
}

impl Provider {
    pub fn open<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let database = Database::open(path.as_ref().join("db").to_str().ok_or_else(|| anyhow!("Invalid level path"))?)?;
        Ok(Self { database })
    }

    pub fn get_biome<I>(&self, coordinates: I, dimension: Dimension) -> anyhow::Result<Option<ChunkBiome>>
    where
        I: Into<Vector<i32, 2>>,
    {
        let key = DataKey {
            coordinates: coordinates.into(),
            dimension,
            data: KeyType::Biome3d,
        };

        if let Some(data) = self.database.get(key.clone())? {
            let biome = ChunkBiome::deserialize(&*data)?;
            Ok(Some(biome))
        } else {
            // Key not found
            Ok(None)
        }
    }
}
