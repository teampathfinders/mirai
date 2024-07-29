use std::ops::Range;

use level::{BiomeEncoding, BiomeStorage, BlockStates, SubChunk};
use util::{BinaryWrite, RVec, Vector};

use crate::level::viewer::ChunkOffset;

pub struct ChunkColumn {
    pub subchunks: Vec<(u16, Option<SubChunk>)>,
    pub biomes: Vec<BiomeEncoding>,
    pub range: Range<i16>,
    coordinates: Vector<i32, 2>,
    /// Column chunk heightmap.
    /// This is pregenerated and used to compute subchunk heightmaps.
    heightmap: Box<[[i16; 16]; 16]>,
    highest_nonempty: u16,
}

impl ChunkColumn {
    pub fn empty(coordinates: Vector<i32, 2>) -> ChunkColumn {
        ChunkColumn {
            coordinates,
            subchunks: Vec::new(),
            range: 0..0,
            heightmap: Box::new([[0; 16]; 16]),
            biomes: Vec::new(),
            highest_nonempty: 0,
        }
    }

    pub fn serialize_biomes(&self) -> anyhow::Result<RVec> {
        todo!()
    }

    pub fn heightmap(&self) -> &Box<[[i16; 16]; 16]> {
        &self.heightmap
    }

    /// Returns the index of the highest subchunk that is nonempty.
    #[inline]
    pub const fn highest_nonempty(&self) -> u16 {
        self.highest_nonempty
    }

    pub fn generate_heightmap(&mut self) {
        for x in 0..16 {
            for z in 0..16 {
                let mut top_coord = 0;
                for (index, sub) in self.subchunks.iter().rev().filter_map(|(index, sub)| sub.as_ref().map(|x| (index, x))) {
                    if self.highest_nonempty != 0 {
                        self.highest_nonempty = *index;
                    }

                    if top_coord != 0 {
                        break;
                    }

                    // Using a 16..0 range does not work...
                    for y in (0..16).rev() {
                        let block = sub.layer(0).unwrap().get((x, y, z)).unwrap();
                        if block.name != "minecraft:air" {
                            top_coord = y as i16 + self.index_to_y(sub.index);
                            break;
                        }
                    }
                }

                self.heightmap[x as usize][z as usize] = top_coord;
            }
        }
    }

    /// Converts a vertical coordinate to a subchunk index in this column.
    pub fn y_to_index(&self, y: i16) -> i8 {
        ((y - self.range.start) / 16) as i8
    }

    pub fn index_to_y(&self, index: i8) -> i16 {
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
