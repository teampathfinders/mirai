use bytes::{BytesMut, BufMut};
use common::{Encodable, VResult};

/// Database key prefixes.
///
/// Data from [`Minecraft fandom`](https://minecraft.fandom.com/wiki/Bedrock_Edition_level_format#Chunk_key_format).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DatabaseTag {
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

#[derive(Debug)]
pub struct DatabaseKey {
    /// X coordinate of the chunk.
    pub x: i32,
    /// Z coordinate of the chunk.
    pub z: i32,
    /// Y coordinate of the chunk.
    /// Only used if the tag is a sub chunk tag.
    pub y: i8,
    /// Dimension of the chunk.
    pub dimension: Dimension,
    /// The tag of the data to load.
    pub tag: DatabaseTag,
}

impl Encodable for DatabaseKey {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(
            4 + 4 + if self.dimension != Dimension::Overworld {
                4
            } else {
                0
            } + 1 + if self.tag == DatabaseTag::SubChunk {
                1
            } else {
                0
            }
        );

        buffer.put_i32_le(self.x);
        buffer.put_i32_le(self.z);

        if self.dimension != Dimension::Overworld {
            buffer.put_i32_le(self.dimension as i32);
        }

        buffer.put_u8(self.tag as u8);
        if self.tag == DatabaseTag::SubChunk {
            buffer.put_i8(self.y);
        }

        Ok(buffer)
    }
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
