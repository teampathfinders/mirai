use std::marker::PhantomData;

use crate::{request::{Req, Requestable}, FilterCollection};

pub trait SystemParam {
    const PARALLEL: bool;
}

impl SystemParam for () {
    const PARALLEL: bool = true;
}

impl<'r, R: Requestable, F: FilterCollection> SystemParam for Req<'r, R, F> {
    const PARALLEL: bool = R::PARALLEL;
}

pub trait SystemParams {
    const PARALLEL: bool;
}

impl<P: SystemParam> SystemParams for P {
    const PARALLEL: bool = <P as SystemParam>::PARALLEL;
}

pub trait System {

}

pub struct ParallelContainer<S, P: SystemParams> {
    runnable: S,
    _marker: PhantomData<P>
}

impl<S, P: SystemParams> ParallelContainer<S, P> {
    pub fn new(runnable: S) -> Self {
        debug_assert!(P::PARALLEL, "Cannot create ParallelContainer from exclusive system");
        Self {
            runnable,
            _marker: PhantomData
        }
    }
}

impl<S, P: SystemParam> System for ParallelContainer<S, P> {
    
}

pub trait IntoSystem<S, P> 
where
    P: SystemParams
{
    fn into_boxed(self) -> Box<dyn System>;
}

impl<S: Fn(P) + 'static, P: SystemParam + 'static> IntoSystem<S, P> for S {
    fn into_boxed(self) -> Box<dyn System> {
        if P::PARALLEL {
            let container: ParallelContainer<S, P> = ParallelContainer::new(self);
            Box::new(container)
        } else {
            todo!();
        }
    }
}

#[derive(Default)]
pub struct Executor {
    
}

impl Executor {
    pub fn new() -> Self {
        Self::default()
    }
}