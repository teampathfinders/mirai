use std::ops::Range;

use level::{BlockStates, SubChunk};
use util::BinaryWrite;

use crate::level::viewer::ChunkOffset;

pub struct ChunkColumn {
    pub subchunks: Vec<(ChunkOffset, Option<SubChunk>)>,
    pub range: Range<i16>,
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

    pub fn heightmap(&self) -> &Box<[[i16; 16]; 16]> {
        &self.heightmap
    }

    pub fn generate_heightmap(&mut self) {
        for x in 0..16 {
            for z in 0..16 {
                let mut top_found = false;

                for sub in self.subchunks.iter().rev().filter_map(|(_, sub)| sub.as_ref()) {
                    if top_found {
                        break;
                    }

                    for y in (0..16).rev() {
                        let block = sub.layer(0).unwrap().get((x, y, z));
                        dbg!(block);
                    }
                }
            }
        }

        // todo!()
    }

    /// Converts a vertical coordinate to a subchunk index in this column.
    pub fn y_to_index(&self, y: i16) -> u16 {
        ((y - self.range.start) / 16) as u16
    }

    pub fn index_to_y(&self, index: u16) -> i16 {
        (index * 16) as i16 + self.range.start
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
