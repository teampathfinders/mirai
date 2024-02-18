use std::{iter::FusedIterator, ops::{Range, RangeInclusive}, pin::Pin, sync::Arc, task::{ready, Context, Poll}};

use level::{provider::Provider, SubChunk};
use rayon::iter::{plumbing::{bridge, Consumer, Producer, ProducerCallback, UnindexedConsumer}, IndexedParallelIterator, ParallelIterator};
use util::Vector;

/// Types that can be used in region requests.
pub trait Region: Clone + Send + Sync + 'static {
    /// Iterator that this region can be turned into.
    type IntoIter: IndexedParallelIterator<Item = Vector<i32, 3>>;

    /// Creates an iterator over this region.
    fn iter(&self, provider: Arc<Provider>) -> Self::IntoIter;
    /// Converts a coordinate to an index into this region.
    fn as_index(&self, coord: &Vector<i32, 3>) -> Option<usize>;
    /// Converts an index to a coordinate into this region.
    fn as_coord(&self, index: usize) -> Option<Vector<i32, 3>>;
    /// Amount of subchunks contained in this region.
    fn len(&self) -> usize;
}

/// Produces split region iterators.
pub struct RegionProducer<T: Region>(RegionIter<T>);

impl<T: Region> Producer for RegionProducer<T> {
    type Item = Vector<i32, 3>;
    type IntoIter = RegionIter<T>;

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

impl<T: Region> From<RegionIter<T>> for RegionProducer<T> {
    #[inline]
    fn from(value: RegionIter<T>) -> Self {
        Self(value)
    }
}

/// An iterator that iterates over every single subchunk coordinate within a region.
pub struct RegionIter<T: Region> {    
    pub(super) region: T,
    pub(super) front_index: usize,
    pub(super) back_index: usize,
    pub(super) provider: Arc<Provider>
}

impl<T: Region> ParallelIterator for RegionIter<T> {
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

impl<T: Region> IndexedParallelIterator for RegionIter<T> {
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

impl<T: Region> Iterator for RegionIter<T> {
    type Item = Vector<i32, 3>;

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.front_index += n;
        self.next()
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.len() > 0 {
            self.front_index += 1;
            self.region.as_coord(self.front_index - 1)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(ExactSizeIterator::len(self)))
    }
}

impl<T: Region> ExactSizeIterator for RegionIter<T> {
    #[inline]
    fn len(&self) -> usize {
        // Use checked subtraction to make sure the length does not overflow
        // when back_index < front_index.
        let len = self.back_index.checked_sub(self.front_index).unwrap_or(0);
        len
    }
}

impl<T: Region> FusedIterator for RegionIter<T> {}

impl<T: Region> DoubleEndedIterator for RegionIter<T> {
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        // Unlike `nth`, this can overflow if we are already at 0.
        self.back_index.checked_sub(n)?;
        self.next_back()
    }
    
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len() > 0 {
            self.back_index -= 1;
            self.region.as_coord(self.back_index + 1)
        } else {
            None
        }
    }
}

impl<T: Region> From<RegionProducer<T>> for RegionIter<T> {
    #[inline]
    fn from(producer: RegionProducer<T>) -> Self {
        producer.0
    }
}