use dashmap::DashMap;

use crate::raknet::Frame;
use crate::raknet::packets::AckRecord;

#[derive(Debug)]
pub struct RecoveryQueue {
    frames: DashMap<u32, Frame>,
}

impl RecoveryQueue {
    pub fn new() -> Self {
        Self {
            frames: DashMap::new(),
        }
    }

    pub fn insert(&self, frame: Frame) {
        self.frames.insert(frame.reliable_index, frame);
    }

    pub fn confirm(&self, records: &Vec<AckRecord>) {
        for record in records {
            match record {
                AckRecord::Single(id) => {
                    self.frames.remove(&id);
                }
                AckRecord::Range(range) => {
                    for id in range.clone() {
                        self.frames.remove(&id);
                    }
                }
            }
        }
    }

    pub fn recover(&self, records: &Vec<AckRecord>) -> Vec<Frame> {
        let mut recovered = Vec::new();
        for record in records {
            match record {
                AckRecord::Single(id) => {
                    if let Some(frame) = self.frames.get(&id) {
                        recovered.push(frame.value().clone());
                    }
                }
                AckRecord::Range(range) => {
                    recovered.reserve(range.len());
                    for id in range.clone() {
                        if let Some(frame) = self.frames.get(&id) {
                            recovered.push(frame.value().clone());
                        }
                    }
                }
            }
        }

        recovered
    }
}
