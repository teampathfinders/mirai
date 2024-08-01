use std::ops::Range;

use level::{BiomeEncoding, BiomeStorage, Biomes, BlockStates, SubChunk};
use util::{BinaryWrite, RVec, Vector};

use crate::level::viewer::ChunkOffset;

pub struct ChunkColumn {
    /// Pair of absolute subchunk indices and (optional) subchunk data.
    /// If there is no data, then the subchunk is full air.
    pub subchunks: Vec<(u16, Option<SubChunk>)>,
    pub biomes: Vec<BiomeEncoding>,
    /// Vertical range of this column in terms of absolute subchunk coordinates.
    pub range: Range<i16>,
    coordinates: Vector<i32, 2>,
    /// Column chunk heightmap.
    /// This is pregenerated and used to compute subchunk heightmaps.
    heightmap: Box<[[i16; 16]; 16]>,
    /// Highest chunk that is nonair relative to the bottom limit of this column.
    /// (that is why the index is always positive unlike absolute subchunk coordinates).
    highest_nonair: u16,
}

impl ChunkColumn {
    pub fn empty(coordinates: Vector<i32, 2>) -> ChunkColumn {
        ChunkColumn {
            coordinates,
            subchunks: Vec::new(),
            range: 0..0,
            heightmap: Box::new([[0; 16]; 16]),
            biomes: Vec::new(),
            highest_nonair: 0,
        }
    }

    pub fn serialize_biomes(&self) -> anyhow::Result<RVec> {
        let mut writer = RVec::alloc();

        for fragment in &self.biomes {
            match fragment {
                BiomeEncoding::Inherit => writer.write_u8(0x7f << 1)?,
                BiomeEncoding::Single(single) => {
                    writer.write_u8(0)?;
                    writer.write_u32_le(*single)?;
                }
                BiomeEncoding::Paletted(biome) => {
                    level::serialize_packed_array(&mut writer, &biome.indices, biome.palette.len(), true)?;

                    writer.write_u32_le(biome.palette.len() as u32)?;
                    for entry in &biome.palette {
                        writer.write_u32_le(*entry)?;
                    }
                }
            }
        }

        Ok(writer)
    }

    pub fn heightmap(&self) -> &Box<[[i16; 16]; 16]> {
        &self.heightmap
    }

    /// Returns the index of the highest subchunk that is nonempty.
    #[inline]
    pub const fn highest_nonair(&self) -> u16 {
        self.highest_nonair
    }

    pub fn generate_heightmap(&mut self) {
        for x in 0..16 {
            for z in 0..16 {
                let mut top_coord = 0;
                for (index, sub) in self.subchunks.iter().rev().filter_map(|(index, sub)| sub.as_ref().map(|x| (index, x))) {
                    if self.highest_nonair < *index {
                        self.highest_nonair = *index;
                    }

                    if top_coord != 0 {
                        break;
                    }

                    // Using a 16..0 range sadly does not work...
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
}
