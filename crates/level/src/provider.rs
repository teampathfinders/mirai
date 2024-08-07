// Special keys

use crate::biome::Biomes;
use crate::database::Database;
use crate::settings::LevelSettings;
use crate::{DataKey, KeyType, SubChunk, WriteBatch};
use anyhow::anyhow;
use proto::types::Dimension;
use std::path::{Path, PathBuf};
use util::BinaryRead;
use util::Vector;

/// Provides world data.
///
/// This is a wrapper around a database that also deserialises and serialises data.
/// It does not implement caching of any kind, that is up to the caller.
pub struct Provider {
    /// Database to load the data from.
    database: Database,
    path: PathBuf,
}

impl Provider {
    /// Opens the specified world.
    ///
    /// # Errors
    ///
    /// This method can fail if the database cannot be opened (it does not exist, it is corrupted, it is already in use, etc.)
    /// It can also fail if the given path is not valid UTF-8.
    pub fn open<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let database = Database::open(path.as_ref().join("db").to_str().ok_or_else(|| anyhow!("Invalid level path"))?)?;
        Ok(Self { database, path: path.as_ref().to_owned() })
    }

    /// Gets the world settings, encoded in the `level.dat` file.
    ///
    /// # Errors
    ///
    /// This method returns an error if the content does not match what is specified in the header.
    #[tracing::instrument(skip_all, name = "Provider::settings")]
    pub fn settings(&self) -> anyhow::Result<LevelSettings> {
        let raw = std::fs::read(self.path.join("level.dat"))?;

        let mut reader = raw.as_slice();
        let _file_version = reader.read_u32_le()?;
        let file_size = reader.read_u32_le()?;

        let remaining = reader.remaining();
        if remaining != file_size as usize {
            tracing::error!("Invalid `level.dat` file: header specified length of {file_size}, but found {remaining}");
            anyhow::bail!("Invalid `level.dat` file: header specified length of {file_size} bytes, but found {remaining}");
        }

        let (settings, _) = nbt::from_le_bytes(&mut reader)?;
        Ok(settings)
    }

    /// Load the version of the specified chunk.
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
    pub fn version<I>(&self, coordinates: I, dimension: Dimension) -> anyhow::Result<Option<u8>>
    where
        I: Into<Vector<i32, 2>>,
    {
        let key = DataKey {
            coordinates: coordinates.into(),
            dimension,
            data: KeyType::ChunkVersion,
        };

        self.database.get(key)?.map_or_else(|| Ok(None), |data| Ok(Some(data[0])))
    }

    /// Load the biomes in the specified chunk.
    ///
    /// See [`Biomes`] for more information.
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
    pub fn biomes<I>(&self, coordinates: I, dimension: Dimension) -> anyhow::Result<Option<Biomes>>
    where
        I: Into<Vector<i32, 2>>,
    {
        let key = DataKey {
            coordinates: coordinates.into(),
            dimension,
            data: KeyType::Biome3d,
        };

        if let Some(data) = self.database.get(key)? {
            let biome = Biomes::deserialize(&*data)?;
            Ok(Some(biome))
        } else {
            Ok(None)
        }
    }

    /// Load the specified sub chunk from the database.
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
    pub fn subchunk<I>(&self, coordinates: I, dimension: Dimension) -> anyhow::Result<Option<SubChunk>>
    where
        I: Into<Vector<i32, 3>>,
    {
        let coordinates = coordinates.into();
        let key = DataKey {
            coordinates: (coordinates.x, coordinates.z).into(),
            dimension,
            data: KeyType::SubChunk { index: coordinates.y as i8 },
        };

        if let Some(data) = self.database.get(key)? {
            let sub_chunk = SubChunk::deserialize_disk(&*data)?;
            Ok(Some(sub_chunk))
        } else {
            Ok(None)
        }
    }

    /// Create a new write batch that can optionally be used in write operations.
    #[inline]
    pub fn batch() -> WriteBatch {
        WriteBatch::new()
    }
}
