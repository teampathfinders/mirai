use util::bytes::{BinaryRead, BinaryWrite};

pub const AUTONOMOUS_ENTITIES: &[u8] = "AutonomousEntities".as_bytes();
pub const BIOME_DATA: &[u8] = "BiomeData".as_bytes();
pub const CHUNK_METADATA: &[u8] = "LevelChunkMetaDataDictionary".as_bytes();
pub const OVERWORLD: &[u8] = "Overworld".as_bytes();
pub const MOB_EVENTS: &[u8] = "mobevents".as_bytes();
pub const SCOREBOARD: &[u8] = "scoreboard".as_bytes();
pub const SCHEDULER: &[u8] = "schedulerWT".as_bytes();
pub const LOCAL_PLAYER: &[u8] = "~local_player".as_bytes();

/// Database key prefixes.
///
/// Data from [`Minecraft fandom`](https://minecraft.fandom.com/wiki/Bedrock_Edition_level_format#Chunk_key_format).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum KeyData {
    /// 3D biome map.
    Biome3d = 0x2b,
    /// Version of the specified chunk.
    ChunkVersion = 0x2c,
    HeightMap = 0x2d,
    /// Sub chunk data.
    SubChunk {
        index: i8,
    } = 0x2f,
    LegacyTerrain = 0x30,
    /// A block entity.
    BlockEntity = 0x31,
    /// An entity.
    Entity = 0x32,
    /// Pending tick data.
    PendingTicks = 0x33,
    /// Biome state.
    BiomeState = 0x35,
    /// Finalized state.
    FinalizedState = 0x36,
    /// Education Edition border blocks.
    BorderBlocks = 0x38,
    /// Bounding boxes for structure spawns stored in binary format.
    HardCodedSpawnAreas = 0x39,
    /// Random tick data.
    RandomTicks = 0x3a,
}

impl KeyData {
    pub fn discriminant(&self) -> u8 {
        // SAFETY: KeyData is marked as `repr(u8)` and therefore its layout is a
        // `repr(C)` union of `repr(C)` structs, each of which has the `u8` discriminant as its first
        // field. Hence, we can read the discriminant without offsetting the pointer.
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseKey {
    /// X coordinate of the chunk.
    pub x: i32,
    /// Z coordinate of the chunk.
    pub z: i32,
    /// Dimension of the chunk.
    pub dimension: Dimension,
    /// The tag of the data to load.
    pub data: KeyData,
}

impl DatabaseKey {
    pub(crate) fn serialized_size(&self) -> usize {
        4 + 4 + if self.dimension != Dimension::Overworld { 4 } else { 0 } + 1 + if let KeyData::SubChunk { .. } = self.data { 1 } else { 0 }
    }

    pub(crate) fn serialize<W>(&self, mut writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite,
    {
        writer.write_i32_le(self.x)?;
        writer.write_i32_le(self.z)?;

        if self.dimension != Dimension::Overworld {
            writer.write_i32_le(self.dimension as i32)?;
        }

        writer.write_u8(self.data.discriminant())?;
        if let KeyData::SubChunk { index } = self.data {
            writer.write_i8(index)?;
        }

        Ok(())
    }

    pub(crate) fn deserialize<'a, R>(mut reader: R) -> anyhow::Result<Self>
    where
        R: BinaryRead<'a> + 'a,
    {
        let x = reader.read_i32_le()?;
        let z = reader.read_i32_le()?;

        let dimension = if reader.remaining() > 6 {
            Dimension::try_from(reader.read_u32_le()?)?
        } else {
            Dimension::Overworld
        };

        let key_ty = reader.read_u8()?;
        let data = match key_ty {
            0x2f => KeyData::SubChunk { index: reader.read_i8()? },
            0x2b => KeyData::Biome3d,
            0x2c => KeyData::ChunkVersion,
            0x2d => KeyData::HeightMap,
            0x30 => KeyData::LegacyTerrain,
            0x31 => KeyData::BlockEntity,
            0x32 => KeyData::Entity,
            0x33 => KeyData::PendingTicks,
            0x35 => KeyData::BiomeState,
            0x36 => KeyData::FinalizedState,
            0x38 => KeyData::BorderBlocks,
            0x39 => KeyData::HardCodedSpawnAreas,
            0x3a => KeyData::RandomTicks,
            _ => anyhow::bail!(format!("Invalid key type: {key_ty:x?}")),
        };

        Ok(Self { x, z, dimension, data })
    }
}

/// The Minecraft dimensions.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Dimension {
    /// The overworld dimension.
    Overworld,
    /// The nether dimension.
    Nether,
    /// The end dimension.
    End,
}

impl TryFrom<u32> for Dimension {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> anyhow::Result<Self> {
        Ok(match value {
            0 => Self::Overworld,
            1 => Self::Nether,
            2 => Self::End,
            _ => anyhow::bail!("Invalid dimension"),
        })
    }
}
