use crate::level::LevelManager;
use crate::network::{HeightmapType, SubChunkEntry, SubChunkResult};
use std::cell::OnceCell;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use util::Vector;

pub struct ChunkViewer {
    radius: AtomicI32,
    level: Arc<LevelManager>,
}

impl ChunkViewer {
    pub fn new(level: Arc<LevelManager>) -> Self {
        Self { radius: AtomicI32::new(0), level }
    }

    #[inline]
    pub fn set_radius(&self, radius: i32) {
        self.radius.store(radius, Ordering::Release);
    }

    #[inline]
    pub fn get_radius(&self) -> i32 {
        self.radius.load(Ordering::Acquire)
    }

    pub fn recenter(&self, center: Vector<i32, 2>, offsets: &[Vector<i8, 3>]) -> anyhow::Result<Vec<SubChunkEntry>> {
        // todo!()

        let mut entries = Vec::with_capacity(offsets.len());
        for offset in offsets {
            // TODO: Check for out of bounds requests

            // if let Some(chunk) = self.level.get_subchunk()? {
            //     todo!();
            // } else {
            //     entries.push(SubChunkEntry {
            //         offset: offset.clone(),
            //         result: SubChunkResult::NotFound,
            //         ..Default::default()
            //     });
            // }

            entries.push(SubChunkEntry {
                offset: offset.clone(),
                result: SubChunkResult::AllAir,
                heightmap_type: HeightmapType::None,
                heightmap: Box::default(),
                payload: vec![],
                blob_hash: 0,
            });
        }

        Ok(entries)
    }
}
