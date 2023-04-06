use std::io::Write;

use level::Dimension;
use util::{Vector, Serialize, bytes::{MutableBuffer, BinaryWrite}};

use crate::network::ConnectedPacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum SubChunkResult {
    Success = 1,
    NotFound,
    InvalidDimension,
    PlayerNotFound,
    OutOfBounds,
    AllAir
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum HeightmapType {
    None,
    WithData,
    TooHigh,
    TooLow
}

#[derive(Debug)]
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
    fn serialize_cached(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        todo!();
    }
    
    #[inline]
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_vecb(&self.offset)?;
        buffer.write_u8(self.result as u8)?;
        buffer.write_all(&self.payload)?;
        buffer.write_u8(self.heightmap_type as u8)?;
        
        if self.heightmap_type == HeightmapType::WithData {
            buffer.write_all(bytemuck::cast_slice(&*self.heightmap))?;
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
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_bool(self.cache_enabled)?;
        buffer.write_var_i32(self.dimension as i32)?;
        buffer.write_veci(&self.position)?;
        
        if self.cache_enabled {
            for entry in &self.entries {
                entry.serialize_cached(buffer)?;
            }   
        } else {
            for entry in &self.entries {
                entry.serialize(buffer)?;
            }
        }
        
        Ok(())
    }
}

