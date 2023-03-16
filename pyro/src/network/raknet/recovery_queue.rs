use dashmap::DashMap;

use crate::network::raknet::packets::AckRecord;
use crate::network::raknet::{FrameBatch};

#[derive(Debug)]
pub struct RecoveryQueue {
    frames: DashMap<u32, FrameBatch>,
}

impl RecoveryQueue {
    pub fn new() -> Self {
        Self { frames: DashMap::new() }
    }

    #[inline]
    pub fn insert(&self, batch: FrameBatch) {
        self.frames.insert(batch.sequence_number, batch);
    }

    pub fn confirm(&self, records: &Vec<AckRecord>) {
        for record in records {
            match record {
                AckRecord::Single(id) => {
                    self.frames.remove(id);
                }
                AckRecord::Range(range) => {
                    for id in range.clone() {
                        self.frames.remove(&id);
                    }
                }
            }
        }
    }

    pub fn recover(&self, records: &Vec<AckRecord>) -> Vec<FrameBatch> {
        let mut recovered = Vec::new();
        for record in records {
            match record {
                AckRecord::Single(id) => {
                    if let Some(frame) = self.frames.remove(id) {
                        recovered.push(frame.1);
                    }
                }
                AckRecord::Range(range) => {
                    recovered.reserve(range.len());
                    for id in range.clone() {
                        if let Some(frame) = self.frames.remove(&id) {
                            recovered.push(frame.1);
                        }
                    }
                }
            }
        }

        recovered
    }
}

impl Default for RecoveryQueue {
    fn default() -> Self {
        Self::new()
    }
}
