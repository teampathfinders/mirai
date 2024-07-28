use std::sync::{
    atomic::{AtomicI32, AtomicU16, Ordering},
    Arc,
};

use futures::{future, StreamExt};
use level::SubChunk;
use proto::{
    bedrock::{HeightmapType, SubChunkEntry, SubChunkResponse, SubChunkResult},
    types::Dimension,
};
use util::Vector;

use super::{BoxRegion, FullChunk, Heightmap, PointRegion, Service};

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

    fn create_entry(&self, base: Vector<i32, 3>, offset: Vector<i8, 3>, full_chunk: &FullChunk) -> anyhow::Result<SubChunkEntry> {
        let absolute_y = base.y + offset.y as i32;
        let subchunk_index = full_chunk.y_to_index(absolute_y);

        let heightmap = Heightmap::new(subchunk_idx, full_chunk);
        let entry = SubChunkEntry {
            result: SubChunkResult::Success,
            offset,
            heightmap_type: HeightmapType::WithDat,
            heightmap,
            blob_hash: todo!(),
            payload: todo!(),
        };

        Ok(entry)
    }

    pub fn load_offsets(&self, base: Vector<i32, 3>, offsets: &[Vector<i8, 3>], dimension: Dimension) -> anyhow::Result<SubChunkResponse> {
        let mut resp = SubChunkResponse {
            dimension,
            entries: Vec::with_capacity(offsets.len()),
            position: base,
            cache_enabled: false,
        };

        for offset in offsets {
            let abs_coord: Vector<i32, 3> = (base.x + offset.x as i32, base.y + offset.y as i32, base.z + offset.z as i32).into();

            match self.load(abs_coord, dimension) {
                Ok(Some(subchunk)) => {
                    if let Ok(entry) = self.create_entry(offset) {
                        resp.entries.push(entry);
                        continue;
                    }
                }
                Ok(None) => {
                    resp.entries.push(SubChunkEntry {
                        result: SubChunkResult::AllAir,
                        ..Default::default()
                    });

                    continue;
                }
                Err(e) => {
                    tracing::error!("Failed to load subchunk at {abs_coord}: {e}");
                }
                _ => {}
            }

            resp.entries.push(SubChunkEntry {
                result: SubChunkResult::NotFound,
                ..Default::default()
            });
        }

        Ok(resp)
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
