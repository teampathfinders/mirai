use std::collections::VecDeque;
use tokio::sync::{Mutex, Semaphore};

/// Queue that supports async blocking pop operations.
pub struct AsyncDeque<T> {
    /// The queue itself.
    deque: Mutex<VecDeque<T>>,
    /// Keeps track of the amount of items currently in the queue.
    permits: Semaphore,
}

impl<T> AsyncDeque<T> {
    /// Creates a new queue with the specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            deque: Mutex::new(VecDeque::with_capacity(capacity)),
            permits: Semaphore::new(0),
        }
    }

    /// Waits for an item to be available and pops it from the queue.
    pub async fn pop(&self) -> T {
        let permit = self.permits
            .acquire()
            .await
            .expect("AsyncDeque semaphore was closed while the queue was still in use");

        permit.forget();

        let mut lock = self.deque.lock().await;
        lock.pop_front().unwrap()
    }

    /// Attempts to pop an item from the queue, returning None if there are no items.
    pub async fn try_pop(&self) -> Option<T> {
        self.permits
            .try_acquire()
            .map_or(None, |p| {
                p.forget();
                Some(())
            })?;

        let mut lock = self.deque.lock().await;
        Some(lock.pop_front().unwrap())
    }

    /// Pushes an item into the queue.
    pub async fn push(&self, item: T) {
        {
            let mut lock = self.deque.lock().await;
            lock.push_back(item);
        }

        self.permits.add_permits(1);
    }
}
