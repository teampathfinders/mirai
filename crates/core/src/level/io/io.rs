use std::{
    pin::Pin,
    sync::Arc,
    task::{ready, Context, Poll},
};

use futures::{Sink, Stream};
use level::SubChunk;
use tokio::sync::{mpsc, watch, Notify};
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
    pub data: SubChunk,
}

/// Streams subchunk data as it is produced by an iterator.
pub struct RegionStream {
    /// Chunk receiver
    pub(super) inner: mpsc::Receiver<IndexedSubChunk>,
    /// Remaining items in the receiver.
    pub(super) len: usize,
}

impl RegionStream {
    /// Returns the remaining length of this stream.
    pub fn len(&self) -> usize {
        self.len
    }
}

impl Stream for RegionStream {
    type Item = IndexedSubChunk;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
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

const FLUSH_THRESHOLD: usize = 10;

/// Collects all subchunk updates and writes them to disk periodically.
struct Collector {
    consumer: mpsc::Receiver<IndexedSubChunk>,
    flush: watch::Sender<()>,

    collection: Vec<IndexedSubChunk>,
}

impl Collector {
    /// Flushes all subchunks in the collector to disk.
    async fn service(mut self) {
        loop {
            let mut spare = Vec::new();
            let Some(chunk) = self.consumer.recv().await else {
                // This collector is no longer referenced, shut it down.
                break;
            };

            self.collection.push(chunk);
            if self.collection.len() >= FLUSH_THRESHOLD {
                std::mem::swap(&mut spare, &mut self.collection);
                Self::begin_flush(spare);
            }
        }
    }

    fn begin_flush(chunks: Vec<IndexedSubChunk>) {
        // Start flushing to disk in a separate thread.
    }
}

/// All unreferenced subchunks are thrown into this sink
/// and will automatically be written to disk at a fixed interval or
/// when the sink is filled up.
pub struct RegionSink {
    producer: mpsc::Sender<IndexedSubChunk>,
    flush: Arc<Notify>,
}

impl Sink<IndexedSubChunk> for RegionSink {
    type Error = anyhow::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<anyhow::Result<()>> {
        // Check whether collector has space.
        // If not, notify it to flush.
        if self.producer.capacity() == 0 {
            self.flush.notify_one();
            return Poll::Pending;
        } else {
        }

        todo!()
    }

    fn start_send(self: Pin<&mut Self>, item: IndexedSubChunk) -> anyhow::Result<()> {
        self.producer.try_send(item)?;
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<anyhow::Result<()>> {
        todo!()
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<anyhow::Result<()>> {
        self.poll_flush(cx)
    }
}
