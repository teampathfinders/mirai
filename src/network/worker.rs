use crate::network::RawPacket;
use crossbeam::deque;

use std::iter;

pub struct Worker<'m> {
    local: deque::Worker<RawPacket>,
    global: &'m deque::Injector<RawPacket>,
    stealers: &'m [deque::Stealer<RawPacket>],
}

impl Worker<'_> {
    fn find_task(&self) -> Option<RawPacket> {
        // Pop a task from the local queue, if not empty
        self.local.pop().or_else(|| {
            // Otherwise, look for a task elsewhere
            iter::repeat_with(|| {
                // Try stealing a batch of tasks from the global queue
                self.global
                    .steal_batch_and_pop(&self.local)
                    // Or try stealing a task from one of the other threads.
                    .or_else(|| self.stealers.iter().map(|s| s.steal()).collect())
            })
            // Loop while no task was stolen and any steal operations need to be retried
            .find(|s| !s.is_retry())
            // Extract the stolen task, if there is one
            .and_then(|s| s.success())
        })
    }
}
