use proto::types::Dimension;
use rayon::iter::IntoParallelIterator;
use util::Vector;

use std::ops::Range;

use super::{Region, RegionIter};

/// A region representing all chunks in a radius around a center.
#[derive(Clone)]
pub struct RadialRegion {
    center: Vector<i32, 3>,
    radius: usize,
    yrange: Range<i32>,
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
        let y = (index % self.len()) as i32 + self.yrange.start;

        let row_size = |row: usize| -> usize {
            2 * (((self.radius.pow(2) - row.pow(2)) as f32).sqrt()).floor() as usize + 1  
        };
        
        let mut count = 0;
        let mut last = 0;
        
        for row in 0..self.radius * 2 + 1 {
            count += row_size((self.radius as i32 - row as i32).abs() as usize);
            if index < count {
                let coord = Vector::from([(index - last) as i32, y, row as i32]);
                return Some(coord)
            }
            last = count;
        }
        
        Some(Vector::from([0, y, 0]))
    }

    fn as_index(&self, coord: &Vector<i32, 3>) -> Option<usize> {
        todo!()
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn len(&self) -> usize {
        // Using the sum from Gauss's circle problem.

        1 + 4 * self.radius + 4 * (1..self.radius)
            .into_iter()
            .map(|i: usize| (((self.radius.pow(2) - i.pow(2)) as f32).sqrt()).floor() as usize)
            .sum::<usize>()
    }
}