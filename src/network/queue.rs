use crossbeam::deque::{Injector, Stealer, Worker};

use std::iter;

pub struct SchedulerQueue<T, const N: usize> {
    injector: Injector<T>,
    stealers: [Stealer<T>; N]
}

impl<T, const N: usize> SchedulerQueue<T, N> {
    pub fn new() -> (SchedulerQueue<T, N>, [Worker<T>; N]) {

        let local_queues: [Worker<T>; N] =
            (0..N).into_iter()
            .map(|_| Worker::new_lifo())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let stealers: [Stealer<T>; N] = local_queues
            .iter()
            .map(|q| q.stealer())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let scheduler = SchedulerQueue {
            injector: Injector::new(),
            stealers
        };

        (scheduler, local_queues)
    }

    pub fn schedule_task(&self, task: T) {
        self.injector.push(task);
    }

    pub fn find_task(&self, dest: &Worker<T>) -> Option<T> {
        iter::repeat_with(|| {
            self.injector.steal_batch_and_pop(dest)
                .or_else(|| self.stealers.iter().map(|s| s.steal()).collect())
        })
        .find(|s| !s.is_retry())
        .and_then(|s| s.success())
    }

}