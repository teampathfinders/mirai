use std::{marker::PhantomData, sync::Arc, mem};

use parking_lot::RwLock;

use crate::{world::WorldState, request::{Requestable, Filters, Request}};

pub unsafe trait SystemParam {
    const READONLY: bool;

    type Target<'state>;

    fn fetch<'state>(state: &'state WorldState) -> Self::Target<'state>;
}   

unsafe impl<S: Requestable, F: Filters> SystemParam for Request<'_, S, F> {
    const READONLY: bool = S::READONLY;

    type Target<'state> = Request<'state, S, F>;

    fn fetch<'state>(state: &'state WorldState) -> Self::Target<'state> {
        Request::new(state)
    }
}

pub trait SystemParams {
    const READONLY: bool;
}

impl<P> SystemParams for P 
where
    P: SystemParam
{
    const READONLY: bool = P::READONLY;
}

pub trait System {
    fn call(&self, state: &WorldState) {
        unimplemented!()
    }

    fn call_mut(&self, state: &mut WorldState) {
        unimplemented!()
    }
}

pub struct ParallelSystem<S, P: SystemParams> 
where
    ParallelSystem<S, P>: System
{
    f: S,
    _marker: PhantomData<P>
}

impl<S, P> ParallelSystem<S, P> 
where
    P: SystemParams,
    ParallelSystem<S, P>: System
{
    pub fn new(f: S) -> Self {
        Self {
            f, _marker: PhantomData
        }
    }
}

impl<S, P> System for ParallelSystem<S, P> 
where
    S: Fn(P),
    P: SystemParam
{
    fn call<'state>(&self, state: &'state WorldState) {
        let fetched = P::fetch(state);
        // (self.f)(fetched);

        // FIXME: state lifetime can be casted to static here, allowing use after free
        let transmuted = unsafe {
            std::mem::transmute_copy(&fetched)
        };
        std::mem::forget(fetched);

        (self.f)(transmuted);
    }
}

pub struct SequentialSystem<S, P> {
    f: S,
    _marker: PhantomData<P>
}

impl<S, P> SequentialSystem<S, P> 
where
    P: SystemParams,
    SequentialSystem<S, P>: System
{
    pub fn new(f: S) -> Self {
        Self {
            f, _marker: PhantomData
        }
    }
}

impl<S, P> System for SequentialSystem<S, P>
where
    S: Fn(P),
    P: SystemParam,
{
    fn call_mut(&self, state: &mut WorldState) {
        todo!();
    }
}

pub trait IntoSystem<S, Params> {
    fn into_boxed(self) -> Arc<dyn System + Send + Sync>;
}

impl<S, P> IntoSystem<S, P> for S 
where
    S: Fn(P) + Send + Sync + 'static,
    P: SystemParam + Send + Sync + 'static
{
    fn into_boxed(self) -> Arc<dyn System + Send + Sync> {
        if P::READONLY {
            Arc::new(ParallelSystem::new(self))
        } else {
            Arc::new(SequentialSystem::new(self))
        }
    }
}

#[derive(Default)]
pub struct Systems {
    parallel: RwLock<Vec<Arc<dyn System + Send + Sync>>>,
    sequential: RwLock<Vec<Arc<dyn System + Send + Sync>>>
}

impl Systems {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<S, P: SystemParams>(&self, system: impl IntoSystem<S, P>) {
        if P::READONLY {
            self.parallel.write().push(system.into_boxed());
        } else {
            self.sequential.write().push(system.into_boxed());
            todo!();
        }
    }

    pub async fn run_all(&self, state: &WorldState) {
        let lock = self.parallel.read();
        for system in &*lock {
            system.call(state);
        }

        // Wait for all systems to finish running
        // while task_set.join_next().await.is_some() {}
    }
}