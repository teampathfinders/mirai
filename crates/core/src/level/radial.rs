use proto::types::Dimension;
use rayon::iter::IntoParallelIterator;
use util::Vector;

use super::{Region, RegionIter};

/// A region representing all chunks in a radius around a center.
#[derive(Clone)]
pub struct RadialRegion {
    center: Vector<i32, 3>,
    radius: u32,
    dimension: Dimension
}

impl IntoIterator for RadialRegion {
    type IntoIter = RegionIter<Self>;
    type Item = Vector<i32, 3>;

    fn into_iter(self) -> Self::IntoIter {
        RegionIter { front_index: 0, back_index: self.len(), region: self }
    }
}

impl IntoParallelIterator for RadialRegion {
    type Iter = RegionIter<Self>;
    type Item = Vector<i32, 3>;

    fn into_par_iter(self) -> Self::Iter {
        RegionIter { front_index: 0, back_index: self.len(), region: self }
    }
}

impl Region for RadialRegion {
    fn as_coord(&self, index: usize) -> Option<Vector<i32, 3>> {
        todo!()
    }

    fn as_index(&self, coord: &Vector<i32, 3>) -> Option<usize> {
        todo!()
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn len(&self) -> usize {
        todo!()
    }
}