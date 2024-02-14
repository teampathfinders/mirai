use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};

use parking_lot::Mutex;

use crate::Frame;

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

/// Contains three LIFO deques that each have a different priority assigned to them.
/// The priority of a given frame determines which queue it enters and how the
/// server prioritizes sending it.
#[derive(Default, Debug)]
pub struct SendQueues {
    /// Queue for high priority frames.
    /// Flushed every session tick.
    high_priority: Mutex<VecDeque<Frame>>,
    /// Queue for medium priority frames.
    /// Flushed every 2 session ticks.
    medium_priority: Mutex<VecDeque<Frame>>,
    /// Queue for low priority frames.
    /// Flushed every 4 session ticks.
    low_priority: Mutex<VecDeque<Frame>>,
    /// It is faster to update a boolean on each read/write and check that,
    /// than to lock each of the three queues to check if they are empty.
    is_empty: AtomicBool,
}

impl SendQueues {
    /// Creates a new send queue.
    #[inline]
    pub fn new() -> SendQueues {
        SendQueues::default()
    }

    /// Whether all three priority queues are completely empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        let empty = self.is_empty.load(Ordering::SeqCst);

        #[cfg(debug_assertions)]
        {
            let hp = self.high_priority.lock().is_empty();
            let mp = self.medium_priority.lock().is_empty();
            let lp = self.low_priority.lock().is_empty();

            debug_assert_eq!(hp && mp && lp, empty, "Empty status does not reflect actual state");
        }

        empty
    }

    /// Inserts a new packet into the send queue.
    pub fn insert_raw(&self, priority: SendPriority, frame: Frame) {
        self.is_empty.store(false, Ordering::SeqCst);

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

    /// Flushes the specified queue.
    pub fn flush(&self, priority: SendPriority) -> Option<Vec<Frame>> {
        // FIXME: This function can potentially return a reference instead of moving the frames
        // to reduce allocations.

        // self.is_empty.store(true, Ordering::SeqCst);
        match priority {
            SendPriority::High => {
                let is_empty = self.low_priority.lock().is_empty() && self.medium_priority.lock().is_empty();
                self.is_empty.store(is_empty, Ordering::SeqCst);

                let mut lock = self.high_priority.lock();
                if lock.is_empty() {
                    None
                } else {
                    Some(lock.drain(0..).collect::<Vec<_>>())
                }
            }
            SendPriority::Medium => {
                let is_empty = self.low_priority.lock().is_empty() && self.medium_priority.lock().is_empty();
                self.is_empty.store(is_empty, Ordering::SeqCst);

                let mut lock = self.medium_priority.lock();
                if lock.is_empty() {
                    None
                } else {
                    Some(lock.drain(0..).collect::<Vec<_>>())
                }
            }
            SendPriority::Low => {
                let is_empty = self.low_priority.lock().is_empty() && self.medium_priority.lock().is_empty();
                self.is_empty.store(is_empty, Ordering::SeqCst);

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