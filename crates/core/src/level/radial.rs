use std::sync::Arc;

use level::provider::Provider;
use util::Vector;

use super::{Region, RegionIter};

#[derive(Clone)]
pub struct RadialRegion {
    center: Vector<i32, 3>,
    radius: u32
}

impl Region for RadialRegion {
    type IntoIter = RegionIter<Self>;

    fn iter(&self, provider: Arc<Provider>) -> Self::IntoIter {
        RegionIter {
            front_index: 0,
            back_index: self.len(),
            provider,
            region: self.clone()
        }
    }

    fn as_coord(&self, index: usize) -> Option<Vector<i32, 3>> {
        todo!()
    }

    fn as_index(&self, coord: &Vector<i32, 3>) -> Option<usize> {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }
}