use std::{pin::Pin, sync::Arc, future::Future, task::{ready, Context, Poll}};

use futures::{Sink, Stream};
use level::SubChunk;
use parking_lot::Mutex;
use tokio::sync::{mpsc, Notify};
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

/// Streams subchunk data as it is produced by the provider.
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

const MAX_SINK_SIZE: usize = 10;

/// Sink that automatically buffers subchunk writes.
pub struct RegionSink {
    buffer: Arc<Mutex<Vec<IndexedSubChunk>>>,
    flush: Notify
}

impl RegionSink {
    fn flush(&self) -> anyhow::Result<()> {
        todo!()
    }
}

impl Sink<IndexedSubChunk> for RegionSink {
    type Error = anyhow::Error;

    fn poll_ready(
        self: Pin<&mut Self>,
        cx: &mut Context
    ) -> Poll<anyhow::Result<()>> {
        {
            let lock = self.buffer.lock();
            if lock.len() >= MAX_SINK_SIZE {

            }
        }

        Poll::Ready(Ok(()))
    }

    fn start_send(
        self: Pin<&mut Self>,
        item: IndexedSubChunk
    ) -> anyhow::Result<()> {
        {
            let mut lock = self.buffer.lock();
            lock.push(item);
        }

        Ok(())
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context
    ) -> Poll<anyhow::Result<()>> {
        todo!()
    }

    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut Context
    ) -> Poll<anyhow::Result<()>> {
        self.poll_flush(cx)
    }
}