use std::{marker::PhantomData, sync::Arc};

use parking_lot::RwLock;

use crate::world::WorldState;

use super::{param::{ParamSet, Param}, systems::Sys};

pub struct ParallelSystem<S, P: ParamSet> 
where
    ParallelSystem<S, P>: Sys
{
    f: S,
    _marker: PhantomData<P>
}

impl<S, P> ParallelSystem<S, P> 
where
    P: ParamSet,
    ParallelSystem<S, P>: Sys
{
    pub fn new(f: S) -> Self {
        Self {
            f, _marker: PhantomData
        }
    }
}

impl<S, P> Sys for ParallelSystem<S, P> 
where
    S: Fn(P),
    P: Param
{
    fn call(&self, state: Arc<RwLock<WorldState>>) {
        let fetched = P::fetch(state);
        (self.f)(fetched);

        // // FIXME: state lifetime can be casted to static here, allowing use after free
        // let transmuted = unsafe {
        //     std::mem::transmute_copy(&fetched)
        // };
        // std::mem::forget(fetched);

        // (self.f)(transmuted);
    }
}