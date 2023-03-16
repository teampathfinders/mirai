use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use dashmap::DashMap;
use crate::Result;

use crate::network::raknet::{Frame};

#[derive(Debug, Default)]
pub struct OrderChannel {
    channel: DashMap<u32, Arc<Frame>>,
    /// Last complete index received from client.
    last_complete: AtomicU32,
    /// Last index assigned by server.
    last_server_index: AtomicU32,
}

impl OrderChannel {
    pub fn new() -> Self {
        Self {
            channel: DashMap::new(),
            last_complete: AtomicU32::new(0),
            last_server_index: AtomicU32::new(0),
        }
    }

    #[inline]
    pub fn get_server_index(&self) -> u32 {
        self.last_server_index.fetch_add(1, Ordering::SeqCst)
    }

    pub fn insert(&self, frame: Arc<Frame>) -> Option<Vec<Frame>> {
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

            let mapped = ready.into_iter().map(|a| Arc::try_unwrap(a).unwrap()).collect::<Vec<_>>();
            Some(mapped)
        } else {
            None
        }
    }
}
