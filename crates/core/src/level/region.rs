use std::{iter::FusedIterator, ops::{Range, RangeInclusive}, pin::Pin, sync::Arc, task::{ready, Context, Poll}};

use level::{provider::Provider, SubChunk};
use rayon::iter::{plumbing::{bridge, Consumer, Producer, ProducerCallback, UnindexedConsumer}, IndexedParallelIterator, ParallelIterator};
use tokio::sync::mpsc;
use util::Vector;
use futures::{Stream, Sink};

use super::{IntoRegion, Service};

/// Produces split region iterators.
pub struct RegionProducer(RegionIter);

impl Producer for RegionProducer {
    type Item = Vector<i32, 3>;
    type IntoIter = RegionIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        RegionIter::from(self)
    }

    #[inline]
    fn split_at(self, index: usize) -> (Self, Self) {
        let left = Self(RegionIter {
            region: self.0.region.clone(),
            front_index: self.0.front_index,
            back_index: self.0.front_index + index,
            provider: Arc::clone(&self.0.provider)
        });

        let right = Self(RegionIter {
            region: self.0.region,
            front_index: self.0.front_index + index,
            back_index: self.0.back_index,
            provider: self.0.provider
        });

        (left, right)
    }
}

impl From<RegionIter> for RegionProducer {
    #[inline]
    fn from(value: RegionIter) -> Self {
        Self(value)
    }
}

/// An iterator that iterates over every single subchunk coordinate within a region.
pub struct RegionIter {    
    region: RegionQuery,
    front_index: usize,
    back_index: usize,
    provider: Arc<Provider>
}

impl ParallelIterator for RegionIter {
    type Item = Vector<i32, 3>;

    #[inline]
    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>
    {
        bridge(self, consumer)
    }

    #[inline]
    fn opt_len(&self) -> Option<usize> {
        Some(self.region.len())
    }
}

impl IndexedParallelIterator for RegionIter {
    #[inline]
    fn with_producer<CB>(self, callback: CB) -> CB::Output 
    where
        CB: ProducerCallback<Self::Item>
    {
        let producer = RegionProducer::from(self);
        callback.callback(producer)
    }

    #[inline]
    fn drive<C>(self, consumer: C) -> C::Result
    where
        C: Consumer<Self::Item>
    {
        bridge(self, consumer)
    }

    #[inline]
    fn len(&self) -> usize {
        self.region.len()
    }
}

impl Iterator for RegionIter {
    type Item = Vector<i32, 3>;

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.front_index += n;
        self.next()
    }

    fn next(&mut self) -> Option<Self::Item> {
        (self.len() > 0).then(|| {
            self.front_index += 1;
            self.region.as_coord_unchecked(self.front_index - 1)
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(ExactSizeIterator::len(self)))
    }
}

impl ExactSizeIterator for RegionIter {
    #[inline]
    fn len(&self) -> usize {
        // Use checked subtraction to make sure the length does not overflow
        // when back_index < front_index.
        let len = self.back_index.checked_sub(self.front_index).unwrap_or(0);
        len
    }
}

impl FusedIterator for RegionIter {}

impl DoubleEndedIterator for RegionIter {
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        // Unlike `nth`, this can overflow if we are already at 0.
        self.back_index.checked_sub(n)?;
        self.next_back()
    }
    
    fn next_back(&mut self) -> Option<Self::Item> {
        (self.len() > 0).then(|| {
            self.back_index -= 1;
            self.region.as_coord_unchecked(self.back_index + 1)
        })
    }
}

impl From<RegionProducer> for RegionIter {
    #[inline]
    fn from(producer: RegionProducer) -> Self {
        producer.0
    }
}

/// A query that requests a certain region of subchunks from the level provider.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionQuery {
    xrange: Range<i32>,
    yrange: Range<i32>,
    zrange: Range<i32>
}

impl RegionQuery {
    /// Creates an iterator over this region using the given level provider.
    pub fn iter(&self, provider: Arc<Provider>) -> RegionIter {
        RegionIter {
            provider,
            front_index: 0,
            back_index: self.len(),
            region: self.clone()
        }
    }

    /// Creates a region query using two corner coordinates.
    /// 
    /// The region will represent the box between these two corners.
    /// The given bounds should be in subchunk coordinates.
    pub fn from_bounds<B1, B2>(bound1: B1, bound2: B2) -> Self 
    where
        B1: Into<Vector<i32, 3>>,
        B2: Into<Vector<i32, 3>>
    {
        let bound1 = bound1.into();
        let bound2 = bound2.into();

        Self::from_bounds_inner(bound1, bound2)
    }

    /// Converts an index to a coordinate within this region, without checking
    /// for bounds.
    /// 
    /// This function is not marked as unsafe because incorrect input does not cause memory unsafety.
    /// Using an index out of bounds will simply return a coordinate outside of the region.
    /// However, the coordinate will likely be incorrect because different regions use incompatible indices.
    pub fn as_coord_unchecked(&self, mut index: usize) -> Vector<i32, 3> {
        let x = index as i32 % (self.xrange.len() as i32) - self.xrange.start;
        index /= self.xrange.len() as usize;

        let y = index as i32 % (self.yrange.len() as i32) - self.yrange.start;
        index /= self.yrange.len() as usize;

        let z = index as i32 - self.zrange.start;

        Vector::from([x, y, z])
    }

    /// Converts a coordinate to an index within this region, without checking 
    /// for bounds.
    /// 
    /// This function is not marked as unsafe because incorrect input does not cause memory unsafety.
    /// Using a coordinate out of bounds will simply return a index outside of the region.
    /// However, the index will likely be incorrect because different regions use incompatible indices.
    pub fn as_index_unchecked(&self, coord: &Vector<i32, 3>) -> usize {
        (coord.x * (self.yrange.len() as i32 * self.zrange.len() as i32) + coord.y * (self.zrange.len() as i32) + coord.z) as usize
    }

    /// Converts a coordinate within this region to an index, ensuring
    /// that the coordinate is not out of bounds.
    pub fn as_index(&self, coord: &Vector<i32, 3>) -> Option<usize> {
        if !self.xrange.contains(&coord.x) || 
            !self.yrange.contains(&coord.y) || 
            !self.zrange.contains(&coord.z) {
            return None
        }

        Some(self.as_index_unchecked(coord))
    }

    /// Converts an index to a coordinate within this region, ensuring
    /// that the index is not out of bounds.
    pub fn as_coord(&self, index: usize) -> Option<Vector<i32, 3>> {
        (index <= self.len()).then(|| self.as_coord_unchecked(index))
    }

    /// The amount of subchunks contained in this region.
    pub fn len(&self) -> usize {
        let len = self.xrange.len() * self.yrange.len() * self.zrange.len();
        tracing::debug!("len = {len}");
        len
    }

    fn from_bounds_inner(bound1: Vector<i32, 3>, bound2: Vector<i32, 3>) -> Self {
        let xmin = std::cmp::min(bound1.x, bound2.x);
        let xmax = std::cmp::max(bound1.x, bound2.x);
        let xrange = xmin..xmax + 1;

        let ymin = std::cmp::min(bound1.y, bound2.y);
        let ymax = std::cmp::max(bound1.y, bound2.y);
        let yrange = ymin..ymax + 1;

        let zmin = std::cmp::min(bound1.z, bound2.z);
        let zmax = std::cmp::max(bound1.z, bound2.z);
        let zrange = zmin..zmax + 1;

        Self {
            xrange, yrange, zrange
        }
    }
}

impl IntoRegion for RegionQuery {
    type IntoIter = RegionIter;

    #[inline]
    fn iter(&self, provider: Arc<Provider>) -> Self::IntoIter {
        self.iter(provider)
    }

    fn as_index(&self, coord: &Vector<i32, 3>) -> usize {
        self.as_index_unchecked(coord)
    }

    fn as_coord(&self, index: usize) -> Vector<i32, 3> {
        self.as_coord_unchecked(index)
    }
}

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