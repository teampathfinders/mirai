use std::ops::Range;

use rayon::iter::IntoParallelIterator;
use util::Vector;

use super::{Region, RegionIter};

/// A region representing the box between two coordinates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoxRegion {
    xrange: Range<i32>,
    yrange: Range<i32>,
    zrange: Range<i32>
}

impl BoxRegion {
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

impl IntoIterator for BoxRegion {
    type IntoIter = RegionIter<Self>;
    type Item = Vector<i32, 3>;

    fn into_iter(self) -> Self::IntoIter {
        RegionIter { front_index: 0, back_index: self.len(), region: self }
    }
}

impl IntoParallelIterator for BoxRegion {
    type Iter = RegionIter<Self>;
    type Item = Vector<i32, 3>;

    fn into_par_iter(self) -> Self::Iter {
        RegionIter { front_index: 0, back_index: self.len(), region: self }
    }
}

impl Region for BoxRegion {
    fn as_index(&self, coord: &Vector<i32, 3>) -> Option<usize> {
        self.as_index(coord)
    }

    fn as_coord(&self, index: usize) -> Option<Vector<i32, 3>> {
        self.as_coord(index)
    }

    fn len(&self) -> usize {
        BoxRegion::len(self)
    }
}