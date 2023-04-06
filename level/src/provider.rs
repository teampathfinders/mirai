// Special keys

use crate::biome::Biome;
use crate::database::Database;
use crate::{DataKey, Dimension, KeyType, SubChunk};
use anyhow::anyhow;
use std::path::Path;
use util::Vector;

/// Provides world data.
///
/// This is a wrapper around a database that also deserialises and serialises data.
/// It does not implement caching of any kind, that is up to the caller.
pub struct Provider {
    /// Database to load the data from.
    database: Database,
}

impl Provider {
    /// Opens the specified world.
    ///
    /// # Errors
    ///
    /// This method can fail if the database cannot be opened (it does not exist, it is corrupted, etc.)
    /// It can also fail if the given path is not valid UTF-8.
    pub fn open<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let database = Database::open(path.as_ref().join("db").to_str().ok_or_else(|| anyhow!("Invalid level path"))?)?;
        Ok(Self { database })
    }

    /// Gets the version of the specified chunk.
    ///
    /// As of writing, the current chunk version is `40`.
    ///
    /// # Arguments
    ///
    /// * `coordinates` - X and Z coordinates of the chunk.
    /// * `dimension` - Dimension the chunk should be retrieved from.
    ///
    /// # Returns
    ///
    /// This method returns `None` if the requested chunk was not found
    /// and an error if the data could not be loaded.
    pub fn get_version<I>(&self, coordinates: I, dimension: Dimension) -> anyhow::Result<Option<u8>>
    where
        I: Into<Vector<i32, 2>>
    {
        let key = DataKey {
            coordinates: coordinates.into(),
            dimension,
            data: KeyType::ChunkVersion
        };

        if let Some(data) = self.database.get(key)? {
            Ok(Some(data[0]))
        } else {
            Ok(None)
        }
    }

    /// Gets the biomes in the specified chunk.
    ///
    /// See [`Biome`] for more information.
    ///
    /// # Arguments
    ///
    /// * `coordinates` - X and Z coordinates of the chunk.
    /// * `dimension` - Dimension the chunk should be retrieved from.
    ///
    /// # Returns
    ///
    /// This method returns `None` if the requested chunk was not found
    /// and an error if the data could not be loaded.
    pub fn get_biome<I>(&self, coordinates: I, dimension: Dimension) -> anyhow::Result<Option<Biome>>
    where
        I: Into<Vector<i32, 2>>,
    {
        let key = DataKey {
            coordinates: coordinates.into(),
            dimension,
            data: KeyType::Biome3d,
        };

        if let Some(data) = self.database.get(key)? {
            let biome = Biome::deserialize(&*data)?;
            Ok(Some(biome))
        } else {
            Ok(None)
        }
    }

    /// Gets the specified sub chunk from the database.
    ///
    /// See [`SubChunk`] for more information.
    ///
    /// # Arguments
    ///
    /// * `coordinates` - X and Z coordinates of the sub chunk.
    /// * `index` - Vertical coordinate of the sub chunk.
    /// * `dimension` - Dimension the chunk should be retrieved from.
    ///
    /// # Returns
    ///
    /// This method returns `None` if the sub chunk was not found
    /// and an error if the data could not be loaded.
    pub fn get_subchunk<I>(&self, coordinates: I, index: i8, dimension: Dimension) -> anyhow::Result<Option<SubChunk>>
    where
        I: Into<Vector<i32, 2>>,
    {
        let key = DataKey {
            coordinates: coordinates.into(),
            dimension,
            data: KeyType::SubChunk { index }
        };

        if let Some(data) = self.database.get(key)? {
            let sub_chunk = SubChunk::deserialize_local(&*data)?;
            Ok(Some(sub_chunk))
        } else {
            Ok(None)
        }
    }
}
