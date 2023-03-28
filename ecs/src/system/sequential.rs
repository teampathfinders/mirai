use std::marker::PhantomData;

use crate::world::WorldState;

use super::{systems::Sys, param::{Param, ParamSet}};

pub struct SequentialSystem<S, P> {
    f: S,
    _marker: PhantomData<P>
}

impl<S, P> SequentialSystem<S, P> 
where
    P: ParamSet,
    SequentialSystem<S, P>: Sys
{
    pub fn new(f: S) -> Self {
        Self {
            f, _marker: PhantomData
        }
    }
}

impl<S, P> Sys for SequentialSystem<S, P>
where
    S: Fn(P), 
    P: Param
{
    fn call_mut(&self, state: &mut WorldState) {
        todo!();
    }
}