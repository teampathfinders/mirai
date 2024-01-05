use crate::level::{LevelManager, SubChunkPosition};

use crate::level::subchunk::NetSubChunk;
use proto::bedrock::{SubChunkEntry, SubChunkResult};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use util::{MutableBuffer, Vector};

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
        let mut entries = Vec::with_capacity(offsets.len());
        for offset in offsets {
            let coords = SubChunkPosition {
                x: center.x + offset.x as i32,
                y: offset.y,
                z: center.y + offset.z as i32,
            };

            if let Some(subchunk) = self.level.get_subchunk(coords)? {
                let subchunk = NetSubChunk::from(subchunk);
                let mut payload = MutableBuffer::new();

                subchunk.serialize_in(&mut payload)?;

                entries.push(SubChunkEntry {
                    offset: offset.clone(),
                    result: SubChunkResult::Success,
                    payload: payload.into_inner(),
                    ..Default::default()
                });
            } else {
                entries.push(SubChunkEntry {
                    offset: offset.clone(),
                    result: SubChunkResult::AllAir,
                    ..Default::default()
                });
            }
        }

        Ok(entries)
    }
}
