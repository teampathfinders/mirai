use core::range::RangeInclusive;

use level::BlockStates;
use proto::bedrock::{SubChunkEntry, SubChunkResult};
use util::BinaryWrite;

pub struct FullChunk {
    subchunks: Vec<Option<SubChunk>>,
    biomes: Biomes,
    range: RangeInclusive<i32>,
}

impl FullChunk {
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
