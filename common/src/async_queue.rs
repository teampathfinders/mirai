use std::collections::VecDeque;

use parking_lot::Mutex;
use tokio::sync::Semaphore;

/// Queue that supports async blocking pop operations.
#[derive(Debug)]
pub struct AsyncDeque<T: Send> {
    /// The queue itself.
    deque: Mutex<VecDeque<T>>,
    /// Keeps track of the amount of items currently in the queue.
    permits: Semaphore,
}

impl<T: Send> AsyncDeque<T> {
    /// Creates a new queue with the specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            deque: Mutex::new(VecDeque::with_capacity(capacity)),
            permits: Semaphore::new(0),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.permits.available_permits() == 0
    }

    /// Waits for an item to be available and pops it from the queue.
    pub async fn pop(&self) -> T {
        let permit = self.permits.acquire().await.expect(
            "AsyncDeque semaphore was closed while the queue was still in use",
        );

        permit.forget();

        let mut lock = self.deque.lock();

        // Safe to unwrap because we can be 100% sure the queue contains an item.
        lock.pop_front()
            .expect("AsyncDeque was empty, but had open permits")
    }

    /// Attempts to pop an item from the queue, returning None if there are no items.
    pub fn try_pop(&self) -> Option<T> {
        self.permits.try_acquire().map_or(None, |p| {
            p.forget();
            Some(())
        })?;

        let mut lock = self.deque.lock();

        // Safe to unwrap because we can be 100% sure that the queue contains an item.
        Some(
            lock.pop_front()
                .expect("AsyncDeque was empty, but had open permits"),
        )
    }

    /// Pushes an item into the queue.
    pub fn push(&self, item: T) {
        {
            let mut lock = self.deque.lock();
            lock.push_back(item);
        }

        self.permits.add_permits(1);
    }
}
