#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum DatabaseKeyPrefix {
    /// 3D biome map.
    Biome3d = 0x2b,
    /// Version of the specified chunk.
    ChunkVersion = 0x2c,
    /// Sub chunk data.
    SubChunk = 0x2f,
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

/// Database key prefixes.
///
/// Data from [`Minecraft fandom`](https://minecraft.fandom.com/wiki/Bedrock_Edition_level_format#Chunk_key_format).
#[derive(Debug)]
#[repr(u8)]
pub enum DatabaseKey {
    /// 3D biome map.
    Biome3d = 0x2b,
    /// Version of the specified chunk.
    ChunkVersion = 0x2c,
    /// Sub chunk data.
    SubChunk(SubChunkPosition),
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

#[derive(Debug)]
pub struct SubChunkPosition {
    pub x: i32,
    pub z: i32,
    pub y: i8,
    pub dimension: Dimension,
}

/// The Minecraft dimensions.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Dimension {
    /// The overworld dimension.
    Overworld,
    /// The nether dimension.
    Nether,
    /// The end dimension.
    End,
}
