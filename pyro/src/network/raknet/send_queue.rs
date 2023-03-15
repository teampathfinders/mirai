use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use parking_lot::Mutex;
use tokio::net::UdpSocket;

use crate::network::raknet::Frame;

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
    // high_priority: Mutex<VecDeque<Frame<'a>>>,
    // medium_priority: Mutex<VecDeque<Frame>>,
    // low_priority: Mutex<VecDeque<Frame>>,

    // Faster to use a single atomic than to try locking all 3 mutexes and checking if they're empty.
    is_empty: AtomicBool,
}

impl SendQueue {
    /// Creates a new send queue.
    pub fn new() -> Self {
        todo!();

        Self {
            // high_priority: Mutex::new(VecDeque::new()),
            // medium_priority: Mutex::new(VecDeque::new()),
            // low_priority: Mutex::new(VecDeque::new()),
            is_empty: AtomicBool::new(true),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.is_empty.load(Ordering::SeqCst)
    }

    /// Inserts a new packet into the send queue.
    pub fn insert_raw(&self, priority: SendPriority, frame: Frame) {
        self.is_empty.store(false, Ordering::SeqCst);

        // match priority {
        //     SendPriority::High => {
        //         let mut lock = self.high_priority.lock();
        //         lock.push_back(frame);
        //     }
        //     SendPriority::Medium => {
        //         let mut lock = self.medium_priority.lock();
        //         lock.push_back(frame);
        //     }
        //     SendPriority::Low => {
        //         let mut lock = self.low_priority.lock();
        //         lock.push_back(frame);
        //     }
        // }
    }

    pub fn flush(&self, priority: SendPriority) -> Option<Vec<Frame>> {
        self.is_empty.store(true, Ordering::SeqCst);

        // match priority {
        //     SendPriority::High => {
        //         let mut lock = self.high_priority.lock();
        //         if lock.is_empty() {
        //             None
        //         } else {
        //             Some(lock.drain(0..).collect::<Vec<_>>())
        //         }
        //     }
        //     SendPriority::Medium => {
        //         let mut lock = self.medium_priority.lock();
        //         if lock.is_empty() {
        //             None
        //         } else {
        //             Some(lock.drain(0..).collect::<Vec<_>>())
        //         }
        //     }
        //     SendPriority::Low => {
        //         let mut lock = self.low_priority.lock();
        //         if lock.is_empty() {
        //             None
        //         } else {
        //             Some(lock.drain(0..).collect::<Vec<_>>())
        //         }
        //     }
        // }
    }
}

impl Default for SendQueue {
    fn default() -> Self {
        Self::new()
    }
}
