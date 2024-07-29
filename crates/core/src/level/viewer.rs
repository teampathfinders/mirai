use std::ops::Range;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicI32, AtomicU16, Ordering},
        Arc,
    },
};

use futures::{future, StreamExt};
use level::SubChunk;
use nohash_hasher::BuildNoHashHasher;
use proto::{
    bedrock::{HeightmapType, LevelChunk, SubChunkEntry, SubChunkRequestMode, SubChunkResponse, SubChunkResult},
    types::Dimension,
};
use util::{RVec, Vector};

use crate::{level::net::ser::NetworkChunkExt, net::BedrockClient};

use super::io::point::PointRegion;
use super::io::r#box::BoxRegion;
use super::net::column::ChunkColumn;
use super::net::heightmap::Heightmap;
use super::Service;

pub type ChunkOffset = Vector<i8, 3>;

const USE_SUBCHUNK_REQUESTS: bool = true;

impl BedrockClient {
    /// Loads chunks around a center point.
    pub fn load_chunks(&self, center: Vector<i32, 2>, dimension: Dimension) -> anyhow::Result<()> {
        // const VERTICAL_RANGE: Range<u16> = 0..16;

        // // Group all subchunks into chunk columns,
        // // with the map indices being two concatenated 32-bit integers representing X and Z coords.
        // let mut col_map: HashMap<i64, ChunkColumn, BuildNoHashHasher<i64>> = HashMap::with_hasher(std::hash::BuildHasherDefault::default());
        // for vertical_offset in VERTICAL_RANGE {
        //     let xz = (base.x as i64) | (base.y as i64) >> 32;
        //     let col = col_map.entry(xz).or_insert_with(|| ChunkColumn::empty(base));

        //     match self.load((base.x, vertical_offset as i32, base.y).into(), dimension) {
        //         Ok(opt) => {
        //             col.subchunks.push((vertical_offset, opt));
        //         }
        //         Err(e) => {
        //             tracing::error!("Failed to load subchunk {vertical_offset} at {base:?}: {e}");
        //             col.subchunks.push((vertical_offset, None));
        //         }
        //     }
        // }

        // // Generate all heightmaps now that the columns are finalised.
        // // TODO: Could maybe benefit from parallelisation depending on the offset count?
        // col_map.values_mut().for_each(ChunkColumn::generate_heightmap);
        //
        const VERTICAL_RANGE: Range<u16> = 0..16;

        let mut column = ChunkColumn::empty(center);
        for y in VERTICAL_RANGE {
            let opt = self.level.provider.subchunk((center.x, y as i32, center.y), dimension)?;
            column.subchunks.push((y, opt));
        }

        if self.supports_cache() {
            self.send_blob_hashes(center)?;
        } else {
        }

        Ok(())
    }

    /// Sends blob hashes of the chunks that the client requested.
    fn send_blob_hashes(&self, coordinates: Vector<i32, 2>, column: &ChunkColumn, dimension: Dimension) -> anyhow::Result<()> {
        use xxhash_rust::xxh64::xxh64;

        if USE_SUBCHUNK_REQUESTS {
            let biomes = column.serialize_biomes()?;
            let hash = xxh64(biomes, 0);

            self.send(LevelChunk {
                dimension,
                coordinates,
                request_mode: SubChunkRequestMode::Limited {
                    highest_subchunk: column.highest_nonempty(),
                },
                blob_hashes: Some(vec![hash]),
                raw_payload: RVec::alloc_from_slice(&[0]),
            })?;
        } else {
            todo!()
        }
    }

    // pub fn load_column(&self, base: Vector<i32, 2>, dimension: Dimension) -> anyhow::Result<()> {
    //     const VERTICAL_RANGE: Range<u16> = 0..16;

    //     // Group all subchunks into chunk columns,
    //     // with the map indices being two concatenated 32-bit integers representing X and Z coords.
    //     let mut col_map: HashMap<i64, ChunkColumn, BuildNoHashHasher<i64>> = HashMap::with_hasher(std::hash::BuildHasherDefault::default());
    //     for vertical_offset in VERTICAL_RANGE {
    //         let xz = (base.x as i64) | (base.y as i64) >> 32;
    //         let col = col_map.entry(xz).or_insert_with(|| ChunkColumn::empty(base));

    //         match self.load((base.x, vertical_offset as i32, base.y).into(), dimension) {
    //             Ok(opt) => {
    //                 col.subchunks.push((vertical_offset, opt));
    //             }
    //             Err(e) => {
    //                 tracing::error!("Failed to load subchunk {vertical_offset} at {base:?}: {e}");
    //                 col.subchunks.push((vertical_offset, None));
    //             }
    //         }
    //     }

    //     // Generate all heightmaps now that the columns are finalised.
    //     // TODO: Could maybe benefit from parallelisation depending on the offset count?
    //     col_map.values_mut().for_each(ChunkColumn::generate_heightmap);

    //     let mut entries = Vec::with_capacity(VERTICAL_RANGE.len() as usize);
    //     for col in col_map.values() {
    //         for (offset, opt) in &col.subchunks {
    //             if let Some(sub) = opt {
    //                 let subchunk_idx = base.y + offset.y as i32 - (col.range.start as i32) / 16;
    //                 dbg!(subchunk_idx);
    //                 let heightmap = Heightmap::new(subchunk_idx as i8, col);
    //                 dbg!(&heightmap);

    //                 let payload = sub.serialize_network(&self.service.instance().block_states)?;
    //                 entries.push(SubChunkEntry {
    //                     offset: (0, *offset as i8, 0).into(),
    //                     result: SubChunkResult::Success,
    //                     heightmap_type: heightmap.map_type,
    //                     heightmap: heightmap.data,
    //                     blob_hash: 0,
    //                     payload,
    //                 });
    //             } else {
    //                 entries.push(SubChunkEntry {
    //                     result: SubChunkResult::AllAir,
    //                     offset: (0, *offset as i8, 0).into(),
    //                     ..Default::default()
    //                 });
    //             }
    //         }
    //     }
    // }
}

pub struct Viewer {
    pub service: Arc<Service>,
    radius: AtomicU16,

    // The current position of this viewer in chunk coordinates.
    current_x: AtomicI32,
    current_z: AtomicI32,
}

impl Viewer {
    /// Creates a new chunk viewer.
    pub const fn new(service: Arc<Service>) -> Viewer {
        Viewer {
            service,
            radius: AtomicU16::new(0),
            current_x: AtomicI32::new(0),
            current_z: AtomicI32::new(0),
        }
    }

    /// Updates the position of this viewer.
    pub fn update_view(&self, position: Vector<f32, 2>) {
        // Transform player coordinates to chunk coordinates.
        let chunk_x = (position.x / 16.0).ceil() as i32;
        let chunk_z = (position.y / 16.0).ceil() as i32;

        self.current_x.store(chunk_x, Ordering::Relaxed);
        self.current_z.store(chunk_z, Ordering::Relaxed);
    }

    /// Updates the render distance of this viewer
    #[inline]
    pub fn update_radius(&self, radius: u16) {
        self.radius.store(radius, Ordering::Relaxed);
    }

    // pub fn load_column(&self, base: Vector<i32, 2>, dimension: Dimension) -> anyhow::Result<SubChunkResponse> {
    //     const VERTICAL_RANGE: Range<u16> = 0..16;

    //     // Group all subchunks into chunk columns,
    //     // with the map indices being two concatenated 32-bit integers representing X and Z coords.
    //     let mut col_map: HashMap<i64, ChunkColumn, BuildNoHashHasher<i64>> = HashMap::with_hasher(std::hash::BuildHasherDefault::default());
    //     for vertical_offset in VERTICAL_RANGE {
    //         let xz = (base.x as i64) | (base.y as i64) >> 32;
    //         let col = col_map.entry(xz).or_insert_with(|| ChunkColumn::empty(base));

    //         match self.load((base.x, vertical_offset as i32, base.y).into(), dimension) {
    //             Ok(opt) => {
    //                 col.subchunks.push((vertical_offset, opt));
    //             }
    //             Err(e) => {
    //                 tracing::error!("Failed to load subchunk {vertical_offset} at {base:?}: {e}");
    //                 col.subchunks.push((vertical_offset, None));
    //             }
    //         }
    //     }

    //     // Generate all heightmaps now that the columns are finalised.
    //     // TODO: Could maybe benefit from parallelisation depending on the offset count?
    //     col_map.values_mut().for_each(ChunkColumn::generate_heightmap);

    //     let mut entries = Vec::with_capacity(VERTICAL_RANGE.len() as usize);
    //     for col in col_map.values() {
    //         for (offset, opt) in &col.subchunks {
    //             if let Some(sub) = opt {
    //                 let subchunk_idx = base.y + offset.y as i32 - (col.range.start as i32) / 16;
    //                 dbg!(subchunk_idx);
    //                 let heightmap = Heightmap::new(subchunk_idx as i8, col);
    //                 dbg!(&heightmap);

    //                 let payload = sub.serialize_network(&self.service.instance().block_states)?;
    //                 entries.push(SubChunkEntry {
    //                     offset: (0, *offset as i8, 0).into(),
    //                     result: SubChunkResult::Success,
    //                     heightmap_type: heightmap.map_type,
    //                     heightmap: heightmap.data,
    //                     blob_hash: 0,
    //                     payload,
    //                 });
    //             } else {
    //                 entries.push(SubChunkEntry {
    //                     result: SubChunkResult::AllAir,
    //                     offset: (0, *offset as i8, 0).into(),
    //                     ..Default::default()
    //                 });
    //             }
    //         }
    //     }

    //     Ok(SubChunkResponse {
    //         cache_enabled: false,
    //         dimension,
    //         entries,
    //         position: (base.x, 0, base.y).into(),
    //     })
    // }

    #[inline]
    pub fn load(&self, pos: Vector<i32, 3>, dimension: Dimension) -> anyhow::Result<Option<SubChunk>> {
        self.service.provider.subchunk(pos, dimension)
    }
}
