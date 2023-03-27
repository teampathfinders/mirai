use std::{marker::PhantomData, sync::Arc};

use parking_lot::RwLock;

use crate::{world::WorldState, request::{Requestable, Filters, Request}};

pub unsafe trait SystemParam {
    const MUTABLE: bool;

    type Target<'state>;

    fn fetch<'state>(state: &'state WorldState) -> Self::Target<'state>;
}   

unsafe impl<S: Requestable, F: Filters> SystemParam for Request<'_, S, F> {
    const MUTABLE: bool = S::MUTABLE;

    type Target<'state> = Request<'state, S, F>;

    fn fetch<'state>(state: &'state WorldState) -> Self::Target<'state> {
        Request::new(state)
    }
}

pub trait SystemParams {
    const MUTABLE: bool;
}

impl<P> SystemParams for P 
where
    P: SystemParam
{
    const MUTABLE: bool = P::MUTABLE;
}

pub trait System {
    fn call(&self, state: &WorldState);
}

pub struct ParallelSystem<S, P: SystemParams> 
where
    ParallelSystem<S, P>: System
{
    f: S,
    _marker: PhantomData<P>
}

impl<S, P: SystemParams> ParallelSystem<S, P> 
where
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
    fn call(&self, state: &WorldState) {
        let fetched = P::fetch(state);
        let transmuted = unsafe {
            std::mem::transmute_copy(&fetched)
        };
        std::mem::forget(fetched);

        (self.f)(transmuted);
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
        if P::MUTABLE {
            todo!();
        } else {
            Arc::new(ParallelSystem::new(self))
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
        if P::MUTABLE {
            todo!();
        } else {
            self.parallel.write().push(system.into_boxed());
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