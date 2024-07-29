use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context, Poll, Waker},
};

use futures::Sink;
use level::provider::Provider;
use parking_lot::Mutex;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use util::Joinable;

use super::stream::IndexedSubChunk;

/// Future that resolves when [`FlushState`] transitions into a busy state.
pub struct Flushing<'state> {
    state: &'state FlushStateInner,
}

impl<'state> Future for Flushing<'state> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        if self.state.is_complete() {
            // Register flush waker
            self.state.busy_wakers.lock().push(cx.waker().clone());
            return Poll::Pending;
        }

        Poll::Ready(())
    }
}

struct FlushStateInner {
    /// Tasks that should be woken when processing finishes.
    idle_wakers: Mutex<Vec<Waker>>,
    /// Tasks that should be woken when the state triggers processing.
    busy_wakers: Mutex<Vec<Waker>>,
    /// Whether the state is currently in idle mode.
    completed: AtomicBool,
}

impl FlushStateInner {
    pub fn is_complete(&self) -> bool {
        self.completed.load(Ordering::SeqCst)
    }
}

/// Token that can be in two states.
///
/// State changes can be awaited.
#[derive(Clone)]
pub struct FlushState {
    inner: Arc<FlushStateInner>,
}

impl FlushState {
    /// Creates a new asynchronous state.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(FlushStateInner {
                idle_wakers: Mutex::new(Vec::new()),
                busy_wakers: Mutex::new(Vec::new()),
                completed: AtomicBool::new(true),
            }),
        }
    }

    /// Transitions the state to idle and wakes all waiting tasks.
    pub fn finish(&self) {
        self.inner.completed.store(true, Ordering::SeqCst);

        let wakers = {
            let mut lock = self.inner.idle_wakers.lock();
            std::mem::take(&mut *lock)
        };

        for waker in wakers {
            waker.wake();
        }
    }

    /// Transitions the state to busy and wakes all waiting tasks.
    pub fn flush(&self) {
        self.inner.completed.store(false, Ordering::SeqCst);

        let wakers = {
            let mut lock = self.inner.busy_wakers.lock();
            std::mem::take(&mut *lock)
        };

        for waker in wakers {
            waker.wake();
        }
    }

    /// Resolves when the state transitions into processing.
    pub fn flushing(&self) -> Flushing {
        Flushing { state: &self.inner }
    }

    // /// Resolves when the state transitions into idle.
    // pub fn idle(&self) -> Idle {
    //     Idle { state: &self.inner }
    // }

    /// Whether the state is currently idle.
    pub fn is_complete(&self) -> bool {
        self.inner.is_complete()
    }
}

impl Default for FlushState {
    fn default() -> Self {
        Self::new()
    }
}

impl Future for FlushState {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        if !self.inner.completed.load(Ordering::SeqCst) {
            self.inner.idle_wakers.lock().push(cx.waker().clone());
            return Poll::Pending;
        }

        Poll::Ready(())
    }
}

/// Collects all subchunk updates and writes them to disk periodically.
pub struct Collector {
    producer: mpsc::Sender<IndexedSubChunk>,
    provider: Arc<Provider>,
    state: FlushState,
    shutdown_token: CancellationToken,
}

impl Collector {
    pub(crate) fn new(provider: Arc<Provider>, instance_token: CancellationToken, collector_size: usize) -> Self {
        let (producer, consumer) = mpsc::channel(collector_size);
        let state = FlushState::new();
        let shutdown_token = CancellationToken::new();

        tokio::spawn(Collector::collection(
            instance_token.clone(),
            shutdown_token.clone(),
            consumer,
            state.clone(),
            collector_size,
        ));

        Self {
            producer,
            provider,
            state,
            shutdown_token,
        }
    }

    /// Creates a new sink that can be used to write into this collector.
    pub fn create_sink(&self) -> RegionSink {
        RegionSink {
            producer: self.producer.clone(),
            state: self.state.clone(),
        }
    }

    async fn collection(
        instance_token: CancellationToken,
        shutdown_token: CancellationToken,
        mut receiver: mpsc::Receiver<IndexedSubChunk>,
        state: FlushState,
        collector_size: usize,
    ) {
        loop {
            tokio::select! {
                _ = state.flushing() => {
                    // Empty channel and collect all changes.
                    let collected = Collector::collect(&mut receiver, collector_size);

                    // Resume normal sink operations.
                    state.finish();

                    Collector::flush(collected);
                },
                _ = instance_token.cancelled() => {
                    shutdown_token.cancel();
                    break
                }
            }
        }

        // Final flush before closing to prevent data loss
        let collected = Collector::collect(&mut receiver, collector_size);
        Collector::flush(collected);

        tracing::info!("Level sink closed");
    }

    #[inline]
    fn collect(receiver: &mut mpsc::Receiver<IndexedSubChunk>, collector_size: usize) -> Vec<IndexedSubChunk> {
        let mut buffered = Vec::with_capacity(collector_size);
        while let Ok(recv) = receiver.try_recv() {
            buffered.push(recv);
        }

        buffered
    }

    fn flush(data: Vec<IndexedSubChunk>) {
        rayon::spawn(|| {
            data.into_par_iter().for_each(|chunk| {});
        });
    }
}

impl Joinable for Collector {
    async fn join(&self) -> anyhow::Result<()> {
        self.shutdown_token.cancelled().await;
        Ok(())
    }
}

/// All unreferenced subchunks are thrown into this sink
/// and will automatically be written to disk at a fixed interval or
/// when the sink is filled up.
pub struct RegionSink {
    producer: mpsc::Sender<IndexedSubChunk>,
    state: FlushState,
}

impl Sink<IndexedSubChunk> for RegionSink {
    type Error = anyhow::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<anyhow::Result<()>> {
        // Check whether collector has space.
        // If not, notify it to flush.
        if self.producer.capacity() == 0 {
            self.state.flush();

            // SAFETY: This is safe because the state objects it not moved while this pin is used.
            let pin = unsafe { Pin::new_unchecked(&mut self.state) };
            return pin.poll(cx).map(|_| Ok(()));
        }

        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: IndexedSubChunk) -> anyhow::Result<()> {
        println!("Send");
        self.producer.try_send(item)?;
        Ok(())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<anyhow::Result<()>> {
        if self.producer.capacity() == self.producer.max_capacity() {
            // Don't flush when the collector is empty.
            return Poll::Ready(Ok(()));
        }

        self.state.flush();

        // SAFETY: The state object is not moved while this pin is active.
        let pin = unsafe { Pin::new_unchecked(&mut self.state) };
        pin.poll(cx).map(|_| Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<anyhow::Result<()>> {
        self.poll_flush(cx)
    }
}
