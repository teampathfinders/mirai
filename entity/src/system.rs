use std::fmt::Debug;
use crate::query::{FilterBundle, Query, QueryBundle};

trait ExclusiveSystem {

}

trait ParallelSystem {

}

pub trait SystemParams {

}

trait SystemParam {

}

impl<'w, Q: QueryBundle, F: FilterBundle> SystemParam for Query<'w, Q, F> {}

impl<P: SystemParam> SystemParams for P {}
impl<P1: SystemParam, P2: SystemParam> SystemParams for (P1, P2) {}
impl<P1: SystemParam, P2: SystemParam, P3: SystemParam> SystemParams for (P1, P2, P3) {}

pub trait System<P: SystemParams> {

}

pub struct Systems {
    pub(crate) exclusive: Vec<Box<dyn ExclusiveSystem>>,
    pub(crate) parallel: Vec<Box<dyn ParallelSystem>>
}

impl Systems {
    pub fn new() -> Self {
        Systems {
            exclusive: Vec::new(),
            parallel: Vec::new()
        }
    }

    pub fn insert<P: SystemParams>(&mut self, system: impl System<P>) {
        todo!()
    }
}
