use std::collections::VecDeque;

use bytes::BytesMut;
use parking_lot::Mutex;

pub enum SendPriority {
    Immediate,
    High,
    Medium,
    Low,
}

#[derive(Debug)]
pub struct SendQueue {
    high_priority: Mutex<VecDeque<BytesMut>>,
    medium_priority: Mutex<VecDeque<BytesMut>>,
    low_priority: Mutex<VecDeque<BytesMut>>,
}

impl SendQueue {
    pub fn new() -> Self {
        Self {
            high_priority: Mutex::new(VecDeque::new()),
            medium_priority: Mutex::new(VecDeque::new()),
            low_priority: Mutex::new(VecDeque::new()),
        }
    }

    pub fn insert(&self, priority: SendPriority, buffer: BytesMut) {
        match priority {
            SendPriority::Immediate => {
                todo!("Immediate send priority");
            }
            SendPriority::High => {
                let mut lock = self.high_priority.lock();
                lock.push_back(buffer);
            }
            SendPriority::Medium => {
                let mut lock = self.medium_priority.lock();
                lock.push_back(buffer);
            }
            SendPriority::Low => {
                let mut lock = self.low_priority.lock();
                lock.push_back(buffer);
            }
        }
    }
}
