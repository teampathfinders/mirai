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
    bedrock::{HeightmapType, SubChunkEntry, SubChunkResponse, SubChunkResult},
    types::Dimension,
};
use util::Vector;

use super::io::point::PointRegion;
use super::io::r#box::BoxRegion;
use super::net::column::ChunkColumn;
use super::net::heightmap::Heightmap;
use super::Service;

pub type ChunkOffset = Vector<i8, 3>;

pub struct Viewer {
    pub service: Arc<Service>,
    radius: AtomicU16,

    // The current position of this viewer in chunk coordinates.
    current_x: AtomicI32,
    current_z: AtomicI32,
}

impl Viewer {
    pub const fn new(service: Arc<Service>) -> Viewer {
        Viewer {
            service,
            radius: AtomicU16::new(0),
            current_x: AtomicI32::new(0),
            current_z: AtomicI32::new(0),
        }
    }

    /// Updates the position of this viewer.
    pub fn update_position(&self, position: Vector<f32, 2>) {
        // Transform player coordinates to chunk coordinates.
        let chunk_x = (position.x / 16.0).ceil() as i32;
        let chunk_z = (position.y / 16.0).ceil() as i32;

        self.current_x.store(chunk_x, Ordering::Relaxed);
        self.current_z.store(chunk_z, Ordering::Relaxed);

        // Update view if required
        self.on_view_update();
    }

    /// Updates the render distance of this viewer
    #[inline]
    pub fn update_radius(&self, radius: u16) {
        self.radius.store(radius, Ordering::Relaxed);
        self.on_view_update();
    }

    fn create_entry(&self, base: Vector<i32, 3>, offset: ChunkOffset, full_chunk: &ChunkColumn) -> anyhow::Result<SubChunkEntry> {
        let absolute_y = base.y + offset.y as i32;
        let subchunk_index = full_chunk.y_to_index(absolute_y as i16);

        let heightmap = Heightmap::new(subchunk_index, full_chunk);
        let entry = SubChunkEntry {
            result: SubChunkResult::Success,
            offset,
            heightmap_type: heightmap.map_type,
            heightmap: heightmap.data,
            blob_hash: todo!(),
            payload: todo!(),
        };

        Ok(entry)
    }

    pub fn load_offsets(&self, base: Vector<i32, 3>, offsets: &[ChunkOffset], dimension: Dimension) -> anyhow::Result<SubChunkResponse> {
        // Group all subchunks into chunk columns,
        // with the map indices being two concatenated 32-bit integers representing X and Z coords.
        let mut col_map: HashMap<i64, ChunkColumn, BuildNoHashHasher<i64>> = HashMap::with_hasher(std::hash::BuildHasherDefault::default());
        for offset in offsets {
            let abs_coord: Vector<i32, 3> = (base.x + offset.x as i32, base.y + offset.y as i32, base.z + offset.z as i32).into();

            let xz = (abs_coord.x as i64) | (abs_coord.z as i64) >> 32;
            let col = col_map.entry(xz).or_insert_with(ChunkColumn::empty);

            match self.load(abs_coord.clone(), dimension) {
                Ok(opt) => {
                    col.subchunks.push((offset.clone(), opt));
                }
                Err(e) => {
                    tracing::error!("Failed to load subchunk at {abs_coord:?}: {e}");
                    col.subchunks.push((offset.clone(), None));
                }
            }
        }

        // Generate all heightmaps now that the columns are finalised.
        // TODO: Could maybe benefit from parallelisation depending on the offset count?
        col_map.values_mut().for_each(ChunkColumn::generate_heightmap);

        let mut entries = Vec::with_capacity(offsets.len());
        for col in col_map.values() {
            for (offset, opt) in &col.subchunks {
                if let Some(sub) = opt {
                    let subchunk_idx = base.y + offset.y as i32 - col.range.start as i32;
                    dbg!(subchunk_idx);
                    let heightmap = Heightmap::new(subchunk_idx as u16, col);
                    dbg!(&heightmap);

                    let payload = todo!();
                    entries.push(SubChunkEntry {
                        offset: offset.clone(),
                        result: SubChunkResult::Success,
                        heightmap_type: heightmap.map_type,
                        heightmap: heightmap.data,
                        blob_hash: 0,
                        payload,
                    });
                } else {
                    entries.push(SubChunkEntry {
                        result: SubChunkResult::AllAir,
                        offset: offset.clone(),
                        ..Default::default()
                    });
                }
            }
        }

        todo!()
    }

    #[inline]
    pub fn load(&self, pos: Vector<i32, 3>, dimension: Dimension) -> anyhow::Result<Option<SubChunk>> {
        self.service.provider.subchunk(pos, dimension)
    }

    fn on_view_update(&self) {
        let x = self.current_x.load(Ordering::Relaxed);
        let z = self.current_z.load(Ordering::Relaxed);

        // // Request the chunk the player is in
        // let stream = self.service.region(BoxRegion::from_bounds(
        //     (x, -4, z), (x, 15, z), Dimension::Overworld
        // ));

        // tokio::spawn(async move {
        //     let fut = stream.take(1).for_each(|res| {
        //         tracing::debug!("{res:?}");

        //         let chunk = res.data;
        //         chunk.serialize_network().unwrap();

        //         future::ready(())
        //     });

        //     fut.await;
        // });
    }
}
