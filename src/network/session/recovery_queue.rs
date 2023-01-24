use dashmap::DashMap;

use crate::network::raknet::frame::FrameBatch;
use crate::network::raknet::packets::acknowledgements::AckRecord;

#[derive(Debug)]
pub struct RecoveryQueue {
    frames: DashMap<u32, FrameBatch>,
}

impl RecoveryQueue {
    pub fn new() -> Self {
        Self {
            frames: DashMap::new(),
        }
    }

    pub fn insert(&self, batch: FrameBatch) {
        self.frames.insert(batch.get_batch_number(), batch);
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
                    if let Some(frame) = self.frames.get(id) {
                        recovered.push((*frame.value()).clone());
                    }
                }
                AckRecord::Range(range) => {
                    recovered.reserve(range.len());
                    for id in range.clone() {
                        if let Some(frame) = self.frames.get(&id) {
                            recovered.push((*frame.value()).clone());
                        }
                    }
                }
            }
        }

        recovered
    }
}
