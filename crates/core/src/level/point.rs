use std::sync::Arc;

use level::provider::Provider;
use util::Vector;

use super::{Region, RegionIter};

/// A region consisting of a set of points. 
/// 
/// This region can be used to request only single chunks from specific coordinates.
#[derive(Clone)]
pub struct PointRegion {
    points: Vec<Vector<i32, 3>>
}

impl PointRegion {
    /// Creates a point region from a set of points.
    pub fn from_points(points: Vec<Vector<i32, 3>>) -> Self {
        Self { points }
    }
}

impl Region for PointRegion {
    type IntoIter = RegionIter<Self>;

    fn iter(&self, provider: Arc<Provider>) -> Self::IntoIter {
        RegionIter {
            region: self.clone(),
            front_index: 0,
            back_index: self.points.len(),
            provider
        }
    }

    fn as_coord(&self, index: usize) -> Option<Vector<i32, 3>> {
        self.points.get(index).cloned()
    }

    fn as_index(&self, coord: &Vector<i32, 3>) -> Option<usize> {
        self.points
            .iter()
            .enumerate()
            .find_map(|(i, item)| (item == coord).then_some(i))
    }

    fn len(&self) -> usize {
        self.points.len()
    }
}