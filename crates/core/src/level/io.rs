use std::{future::Future, ops::DerefMut, pin::Pin, sync::{atomic::{AtomicBool, Ordering}, Arc}, task::{ready, Context, Poll, Waker}};

use futures::{Sink, SinkExt, Stream};
use level::SubChunk;
use parking_lot::Mutex;
use tokio::sync::{mpsc, oneshot, watch, Notify, Semaphore};
use tokio_util::sync::CancellationToken;
use util::Vector;

/// A unique identifier for a specific subchunk.
/// 
/// First 6 bits are the vertical index, 
/// then 29 bits for the x-coordinate
/// and 29 bits for the z-coordinate.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct RegionIndex(u64);

impl From<Vector<i32, 3>> for RegionIndex {
    fn from(value: Vector<i32, 3>) -> Self {
        const XZ_MASK: u64 = 2u64.pow(29) - 1;

        assert!((value.y as u64) < 63, "Region Y-coordinate too large");
        assert!((value.x as u64) < XZ_MASK, "Region X-coordinate too large");
        assert!((value.z as u64) < XZ_MASK, "Region Z-coordinate too large");

        let mut index = (value.y as u64) << 58;
        index |= ((value.x as u64) & XZ_MASK) << 29;
        index |= (value.z as u64) & XZ_MASK;

        RegionIndex(index)
    }
}

impl From<RegionIndex> for Vector<i32, 3> {
    fn from(value: RegionIndex) -> Self {
        const XZ_MASK: u64 = 2u64.pow(29) - 1;

        let index = value.0;
        let y = (index >> 58) as i32;
        let x = ((index >> 29) & XZ_MASK) as i32;
        let z = (index & XZ_MASK) as i32;

        Vector::from([x, y, z])
    }
}

/// A subchunk with an added index into its owning region.
#[derive(Debug)]
pub struct IndexedSubChunk {
    /// The region index.
    pub index: RegionIndex,
    /// The subchunk data.
    pub data: SubChunk
}

/// Streams subchunk data as it is produced by an iterator.
pub struct RegionStream {
    /// Chunk receiver
    pub(super) inner: mpsc::Receiver<IndexedSubChunk>,
    /// Remaining items in the receiver.
    pub(super) len: usize
}

impl RegionStream {
    /// Returns the remaining length of this stream.
    pub fn len(&self) -> usize {
        self.len
    }
}

impl Stream for RegionStream {
    type Item = IndexedSubChunk;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context
    ) -> Poll<Option<Self::Item>> {
        let poll = self.inner.poll_recv(cx);
        let ready = ready!(poll);

        if ready.is_some() {
            self.len -= 1;
        }

        Poll::Ready(ready)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.len))
    }
}

/// Future that resolves when [`AsyncState`] transitions into a busy state.
pub struct Begin<'state> {
    state: &'state FlushStateInner
}

impl<'state> Future for Begin<'state> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        if self.state.is_idle() {
            // Register flush waker
            self.state.busy_wakers.lock().push(cx.waker().clone());
            return Poll::Pending
        }

        Poll::Ready(())
    }
}

// /// Future that resolves when [`AsyncState`] transitions into a completed state.
// pub struct Begin<'state> {
//     state: &'state FlushStateInner
// }

// impl<'state> Future for Begin<'state> {
//     type Output = ();

//     fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
//         if !self.state.is_idle() {
//             // Register complete waker
//             self.state.idle_wakers.lock().push(cx.waker().clone());
//             return Poll::Pending
//         }

//         Poll::Ready(())
//     }
// }

struct FlushStateInner {
    /// Tasks that should be woken when processing finishes.
    idle_wakers: Mutex<Vec<Waker>>,
    /// Tasks that should be woken when the state triggers processing.
    busy_wakers: Mutex<Vec<Waker>>,
    /// Whether the state is currently in idle mode.
    idle: AtomicBool
}

impl FlushStateInner {
    pub fn is_idle(&self) -> bool {
        self.idle.load(Ordering::SeqCst)
    }
}

/// Token that can be in two states.
/// 
/// State changes can be awaited.
#[derive(Clone)]
pub struct FlushState {
    inner: Arc<FlushStateInner>
}

impl FlushState {
    /// Creates a new asynchronous state.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(FlushStateInner {
                idle_wakers: Mutex::new(Vec::new()),
                busy_wakers: Mutex::new(Vec::new()),
                idle: AtomicBool::new(true)
            })
        }
    }

    /// Transitions the state to idle and wakes all waiting tasks.
    pub fn finish(&self) {
        self.inner.idle.store(true, Ordering::SeqCst);
        
        let wakers = {
            let mut lock = self.inner.idle_wakers.lock();
            std::mem::take(&mut *lock)
        };

        println!("Transition to idle, waking {} waiters", wakers.len());
        for waker in wakers {
            waker.wake();
        }
    }

    /// Transitions the state to busy and wakes all waiting tasks.
    pub fn begin(&self) {
        self.inner.idle.store(false, Ordering::SeqCst);

        let wakers = {
            let mut lock = self.inner.busy_wakers.lock();
            std::mem::take(&mut *lock)
        };

        println!("Transition to busy, waking {} waiters", wakers.len());
        for waker in wakers {
            waker.wake();
        }
    }

    /// Resolves when the state transitions into processing.
    pub fn busy(&self) -> Begin {
        Begin { state: &self.inner }
    }   

    // /// Resolves when the state transitions into idle.
    // pub fn idle(&self) -> Idle {
    //     Idle { state: &self.inner }
    // }

    /// Whether the state is currently idle.
    pub fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }

    /// Triggers a flush.
    pub fn flush(&self) {
        println!("Flush triggered");
        self.inner.idle.store(false, Ordering::SeqCst);
    }
}

impl Future for FlushState {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        if !self.inner.idle.load(Ordering::SeqCst) {
            self.inner.idle_wakers.lock().push(cx.waker().clone());
            return Poll::Pending
        }

        Poll::Ready(())
    }
}

/// Collects all subchunk updates and writes them to disk periodically.
pub struct Collector {
    producer: mpsc::Sender<IndexedSubChunk>,
    state: FlushState
}

impl Collector {
    pub(crate) fn new(
        instance_token: CancellationToken,
        collector_size: usize
    ) -> Self {
        let (producer, consumer) = mpsc::channel(collector_size);
        let state = FlushState::new();

        tokio::spawn(Self::collection(instance_token, consumer, state.clone()));

        Self {
            producer, state
        }
    }

    /// Creates a new sink that can be used to write into this collector.
    pub fn create_sink(&self) -> RegionSink {
        RegionSink {
            producer: self.producer.clone(),
            state: self.state.clone()
        }
    }

    async fn collection(
        instance_token: CancellationToken,
        mut receiver: mpsc::Receiver<IndexedSubChunk>, 
        state: FlushState
    ) {
        loop {
            tokio::select! {
                _ = state.busy() => {
                    while let Ok(recv) = receiver.try_recv() {

                    }
                    state.finish();
                },
                _ = instance_token.cancelled() => break
            }
        }   
    }
}

/// All unreferenced subchunks are thrown into this sink
/// and will automatically be written to disk at a fixed interval or
/// when the sink is filled up.
pub struct RegionSink {
    producer: mpsc::Sender<IndexedSubChunk>,
    state: FlushState
}

impl Sink<IndexedSubChunk> for RegionSink {
    type Error = anyhow::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<anyhow::Result<()>> {
        // Check whether collector has space.
        // If not, notify it to flush.
        if self.producer.capacity() == 0 {
            self.state.begin();

            let pin = unsafe { Pin::new_unchecked(&mut self.state) };
            return pin.poll(cx).map(|_| Ok(()))
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
            return Poll::Ready(Ok(()))
        }

        self.state.begin();

        let pin = unsafe { Pin::new_unchecked(&mut self.state) };
        pin.poll(cx).map(|_| Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<anyhow::Result<()>> {
        self.poll_flush(cx)
    }
}