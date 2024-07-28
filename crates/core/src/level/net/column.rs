use core::range::RangeInclusive;

use level::{Biomes, BlockStates};
use proto::bedrock::{SubChunkEntry, SubChunkResult};
use util::BinaryWrite;

use crate::level::ChunkOffset;

pub struct ChunkColumn {
    pub subchunks: Vec<(ChunkOffset, Option<SubChunk>)>,
    pub range: RangeInclusive<i32>,
    heightmap: Box<[[i16; 16]; 16]>,
}

impl ChunkColumn {
    pub fn empty() -> ChunkColumn {
        ChunkColumn {
            subchunks: Vec::new(),
            range: 0..0,
            heightmap: Box::new([[0; 16]; 16]),
        }
    }

    pub fn generate_heightmap(&mut self) {
        for x in 0..16 {
            for z in 0..16 {
                todo!()
            }
        }

        todo!()
    }

    pub fn y_to_index(&self, y: i16) -> u16 {
        todo!()
    }

    pub fn index_to_y(&self, index: u16) -> i16 {
        todo!()
    }

    fn serialize_network_in<W>(&self, states: &BlockStates, mut writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite,
    {
        // let entries = Vec::with_capacity(self.subchunks.len());
        // for subchunk in &self.subchunks {
        //     if let Some(subchunk) = subchunk {
        //     } else {
        //         entries.push(SubChunkEntry { result: SubChunkResult::NotFound });
        //     }
        // }

        todo!()
    }
}
