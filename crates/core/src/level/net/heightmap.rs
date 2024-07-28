use proto::bedrock::HeightmapType;

#[derive(Debug, Clone)]
pub struct Heightmap {
    pub data: Option<Box<[[i8; 16]; 16]>>,
    pub map_type: HeightmapType,
}

impl Heightmap {
    /// Creates a new heightmap for the given subchunk.
    pub fn new(subchunk_idx: u16, full_chunk: &FullChunk) -> Heightmap {
        let mut heightmap = Box::new([[0; 16]; 16]);

        // Whether at least one of the columns has a topmost block that lies below this subchunk.
        let mut above_top = false;
        // Whether at least one of the columns has a topmost block that lies above this subchunk.
        let mut below_top = false;

        for x in 0..16 {
            for z in 0..16 {
                // Index of coordinate in current subchunk.
                let block_idx = (z as u16) << 4 | (x as u16);
                // Y-coordinate of highest block in column.
                let y = full_chunk.heightmap().at(x, z);
                // Index of subchunk that the highest block is located in.
                let other_idx = full_chunk.y_to_index(y);

                if subchunk_idx > other_idx {
                    // Topmost block is located below current subchunk.
                    heightmap[block_idx] = -1;
                    above_top = true;
                } else if subchunk_idx < other_idx {
                    // Topmost block is located above current subchunk.
                    heightmap[block_idx] = 16;
                    below_top = true;
                } else {
                    // Topmost block is located in current subchunk.
                    heightmap[block_idx] = y - full_chunk.index_to_y(other_idx);
                    above_top = true;
                    below_top = true;
                }
            }
        }

        let mut map_type = HeightmapType::WithData;
        if !above_top {
            // All topmost blocks in this chunk column are located above this subchunk,
            // there is no point in sending heightmap data.
            map_type = HeightmapType::TooLow;
        } else if !below_top {
            // All topmost blocks in this chunk column are located below this subchunk,
            // there is no point in sending heightmap data.
            map_type = HeightmapType::TooHigh;
        }

        Heightmap {
            map_type,
            data: match map_type {
                HeightmapType::WithData => Some(heightmap),
                _ => None,
            },
        }
    }
}
