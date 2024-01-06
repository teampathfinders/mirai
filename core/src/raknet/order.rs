use std::sync::atomic::{AtomicU32, Ordering};

use dashmap::DashMap;

use crate::raknet::Frame;

/// Ensures that frames are processed in the correct order.
///
/// Frames that are marked as ordered, should be pushed into this channel.
/// The channel makes sure that old raknet are received before processing further ones.
/// It also puts the received frames into the correct order.
#[derive(Default, Debug)]
pub struct OrderChannel {
    channel: DashMap<u32, Frame>,
    /// Last complete index received from client.
    last_complete: AtomicU32,
    /// Next index to be used by the server.
    next_index: AtomicU32,
}

impl OrderChannel {
    /// Creates a new order channel.
    pub fn new() -> Self {
        Default::default()
    }

    /// Fetches a new index to assign to an ordered frame.
    ///
    /// Every time this is called, the index is increased by 1.
    #[inline]
    pub fn alloc_index(&self) -> u32 {
        self.next_index.fetch_add(1, Ordering::SeqCst)
    }

    /// Inserts a frame into the order channel.
    ///
    /// In case a sequence of frames is completed, the ready frames will be returned.
    pub fn insert(&self, frame: Frame) -> Option<Vec<Frame>> {
        // FIXME: Return some kind of status code to indicate missing raknet.
        // This should be returned when misses have occurred multiple consecutive times
        // and triggers a NAK to be sent.
        // This mechanism might have to work using sequence numbers though.

        self.channel.insert(frame.order_index, frame);

        // Figure out which indexes are ready.
        let old_index = self.last_complete.load(Ordering::SeqCst);
        let mut current_index = old_index;
        loop {
            if self.channel.contains_key(&current_index) {
                current_index += 1;
            } else {
                break;
            }
        }
        self.last_complete.store(current_index, Ordering::SeqCst);

        let ready_count = current_index - old_index;
        if ready_count != 0 {
            let mut ready = Vec::with_capacity(ready_count as usize);
            for i in old_index..current_index {
                ready.push(
                    self.channel
                        .remove(&i)
                        .expect("Packet not found in order channel")
                        .1,
                );
            }

            Some(ready)
        } else {
            None
        }
    }
}
