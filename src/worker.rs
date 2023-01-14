use crate::error::VexResult;
use crate::raknet::packets::RawPacket;
use crate::util::AsyncDeque;
use std::iter;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::task;
use tokio_util::sync::CancellationToken;

pub const GAME_TICK: Duration = Duration::from_millis(1000 / 20);

static ATOMIC_ID_COUNTER: AtomicU16 = AtomicU16::new(1);

pub struct Worker {
    token: CancellationToken,
    incoming_queue: Arc<AsyncDeque<RawPacket>>,
    leaving_queue: Arc<AsyncDeque<RawPacket>>,

    worker_id: u16,
}

impl Worker {
    pub fn new(
        token: CancellationToken,
        incoming_queue: Arc<AsyncDeque<RawPacket>>,
        leaving_queue: Arc<AsyncDeque<RawPacket>>,
    ) -> task::JoinHandle<()> {
        let worker_id = ATOMIC_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

        let worker = Self {
            token,
            incoming_queue,
            leaving_queue,
            worker_id,
        };

        tokio::spawn(async move { worker.work().await })
    }

    async fn work(&self) {
        let mut start_timestamp;
        loop {
            start_timestamp = Instant::now();

            match tokio::select! {
                _ = self.token.cancelled() => break,
                task = self.find_task() => self.handle_task(task).await
            } {
                Ok(_) => (),
                Err(e) => {
                    tracing::error!("Failed to process packet: {e:?}");
                }
            }

            let time_elapsed = Instant::now().duration_since(start_timestamp);
            if time_elapsed < GAME_TICK {
                tokio::time::sleep(GAME_TICK - time_elapsed).await;
            }
        }

        tracing::info!("Worker {} shut down", self.worker_id);
    }

    async fn handle_task(&self, task: RawPacket) -> VexResult<()> {
        tracing::info!("Worker {} handling task", self.worker_id);

        Ok(())
    }

    async fn find_task(&self) -> RawPacket {
        self.incoming_queue.pop().await
    }
}
