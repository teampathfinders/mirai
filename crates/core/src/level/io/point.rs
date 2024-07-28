use proto::types::Dimension;
use rayon::iter::IntoParallelIterator;
use util::Vector;

use super::{Region, RegionIter};

/// A region consisting of a set of points. 
/// 
/// This region can be used to request only single chunks from specific coordinates.
#[derive(Clone)]
pub struct PointRegion {
    points: Vec<Vector<i32, 3>>,
    dimension: Dimension
}

impl PointRegion {
    /// Creates a point region from a set of points.
    pub fn from_points(points: Vec<Vector<i32, 3>>, dimension: Dimension) -> Self {
        Self { points, dimension }
    }
}

impl IntoIterator for PointRegion {
    type IntoIter = RegionIter<Self>;
    type Item = Vector<i32, 3>;

    fn into_iter(self) -> Self::IntoIter {
        RegionIter { front_index: 0, back_index: self.len(), region: self }
    }
}

impl IntoParallelIterator for PointRegion {
    type Iter = RegionIter<Self>;
    type Item = Vector<i32, 3>;

    fn into_par_iter(self) -> Self::Iter {
        RegionIter { front_index: 0, back_index: self.len(), region: self }
    }
}

impl Region for PointRegion {
    fn as_coord(&self, index: usize) -> Option<Vector<i32, 3>> {
        self.points.get(index).cloned()
    }

    fn as_index(&self, coord: &Vector<i32, 3>) -> Option<usize> {
        self.points
            .iter()
            .enumerate()
            .find_map(|(i, item)| (item == coord).then_some(i))
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn len(&self) -> usize {
        self.points.len()
    }
}