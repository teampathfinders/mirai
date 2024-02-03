use std::io::Write;

use util::{Vector, Serialize, BinaryWrite};

use crate::bedrock::ConnectedPacket;
use crate::types::Dimension;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum SubChunkResult {
    #[default]
    Success = 1,
    NotFound = 2,
    InvalidDimension = 3,
    PlayerNotFound = 4,
    OutOfBounds = 5,
    AllAir = 6
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum HeightmapType {
    #[default]
    None,
    WithData,
    TooHigh,
    TooLow
}

#[derive(Debug, Default)]
pub struct SubChunkEntry {
    pub offset: Vector<i8, 3>,
    pub result: SubChunkResult,
    pub payload: Vec<u8>,
    pub heightmap_type: HeightmapType,
    pub heightmap: Box<[[u16; 16]; 16]>,
    pub blob_hash: u64
}

impl SubChunkEntry {
    #[inline]
    fn serialize_cached<W: BinaryWrite>(&self, _writer: &mut W) -> anyhow::Result<()> {
        todo!();
    }
    
    #[inline]
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_vecb(&self.offset)?;
        writer.write_u8(self.result as u8)?;
        writer.write_var_u32(self.payload.len() as u32)?;
        writer.write_all(&self.payload)?;
        writer.write_u8(self.heightmap_type as u8)?;
        if self.heightmap_type == HeightmapType::WithData {
            writer.write_all(bytemuck::cast_slice(&*self.heightmap))?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct SubChunkResponse {
    pub cache_enabled: bool,
    pub dimension: Dimension,
    pub position: Vector<i32, 3>,
    pub entries: Vec<SubChunkEntry>
}

impl ConnectedPacket for SubChunkResponse {
    const ID: u32 = 0xae;
}

impl Serialize for SubChunkResponse {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_bool(self.cache_enabled)?;
        writer.write_var_i32(self.dimension as i32)?;
        writer.write_veci(&self.position)?;

        writer.write_u32_le(self.entries.len() as u32)?;
        if self.cache_enabled {
            for entry in &self.entries {
                entry.serialize_cached(writer)?;
            }   
        } else {
            for entry in &self.entries {
                entry.serialize_into(writer)?;
            }
        }
        
        Ok(())
    }
}

