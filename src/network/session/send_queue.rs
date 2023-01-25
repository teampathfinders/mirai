use std::collections::VecDeque;
use std::sync::Arc;

use parking_lot::Mutex;
use tokio::net::UdpSocket;

use crate::network::raknet::frame::Frame;

/// Priority of the packet.
/// This affects when they're sent.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SendPriority {
    /// High priority is flushed every session tick.
    High,
    /// Medium priority is flushed every two session ticks.
    Medium,
    /// Low priority is flushed every four session ticks.
    Low,
}

/// Queue containing packets that need to be sent.
/// The packets are sorted by priority.
#[derive(Debug)]
pub struct SendQueue {
    high_priority: Mutex<VecDeque<Frame>>,
    medium_priority: Mutex<VecDeque<Frame>>,
    low_priority: Mutex<VecDeque<Frame>>,
}

impl SendQueue {
    /// Creates a new send queue.
    pub fn new() -> Self {
        Self {
            high_priority: Mutex::new(VecDeque::new()),
            medium_priority: Mutex::new(VecDeque::new()),
            low_priority: Mutex::new(VecDeque::new()),
        }
    }

    /// Inserts a new packet into the send queue.
    pub fn insert(&self, priority: SendPriority, frame: Frame) {
        match priority {
            SendPriority::High => {
                let mut lock = self.high_priority.lock();
                lock.push_back(frame);
            }
            SendPriority::Medium => {
                let mut lock = self.medium_priority.lock();
                lock.push_back(frame);
            }
            SendPriority::Low => {
                let mut lock = self.low_priority.lock();
                lock.push_back(frame);
            }
        }
    }

    pub fn insert_batch(&self, priority: SendPriority, batch: Vec<Frame>) {
        match priority {
            SendPriority::High => {
                let mut deque = batch.into();
                let mut lock = self.high_priority.lock();
                lock.append(&mut deque);
            }
            SendPriority::Medium => {
                let mut deque = batch.into();
                let mut lock = self.medium_priority.lock();
                lock.append(&mut deque);
            }
            SendPriority::Low => {
                let mut deque = batch.into();
                let mut lock = self.low_priority.lock();
                lock.append(&mut deque);
            }
        }
    }

    pub fn flush(&self, priority: SendPriority) -> Option<Vec<Frame>> {
        match priority {
            SendPriority::High => {
                let mut lock = self.high_priority.lock();
                if lock.is_empty() {
                    None
                } else {
                    Some(lock.drain(0..).collect::<Vec<_>>())
                }
            }
            SendPriority::Medium => {
                let mut lock = self.medium_priority.lock();
                if lock.is_empty() {
                    None
                } else {
                    Some(lock.drain(0..).collect::<Vec<_>>())
                }
            }
            SendPriority::Low => {
                let mut lock = self.low_priority.lock();
                if lock.is_empty() {
                    None
                } else {
                    Some(lock.drain(0..).collect::<Vec<_>>())
                }
            }
        }
    }
}
