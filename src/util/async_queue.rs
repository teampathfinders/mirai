use std::collections::VecDeque;
use tokio::sync::{Mutex, Semaphore};

pub struct AsyncDeque<T> {
    deque: Mutex<VecDeque<T>>,
    permits: Semaphore,
}

impl<T> AsyncDeque<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            deque: Mutex::new(VecDeque::with_capacity(capacity)),
            permits: Semaphore::new(0),
        }
    }

    pub async fn pop(&self) -> T {
        let permit = self.permits.acquire().await.unwrap();
        permit.forget();

        let mut lock = self.deque.lock().await;
        lock.pop_front().unwrap()
    }

    pub async fn push(&self, item: T) {
        {
            let mut lock = self.deque.lock().await;
            lock.push_back(item);
        }

        self.permits.add_permits(1);
    }
}
