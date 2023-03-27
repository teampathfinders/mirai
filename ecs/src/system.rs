use std::{marker::PhantomData, sync::Arc};

use parking_lot::RwLock;
use tokio::task::JoinSet;

use crate::{world::WorldState, request::{Requestable, Filters, Request}};

pub trait SystemParam {
    const MUTABLE: bool;

    fn fetch(state: Arc<RwLock<WorldState>>) -> Self;
}   

impl<S: Requestable, F: Filters> SystemParam for Request<'_, S, F> {
    const MUTABLE: bool = S::MUTABLE;

    fn fetch(state: Arc<RwLock<WorldState>>) -> Self {
        todo!()
        // Request::new(state)
    }
}

pub trait SystemParams {
    const MUTABLE: bool;
}

impl<P: SystemParam> SystemParams for P {
    const MUTABLE: bool = P::MUTABLE;
}

impl<P0: SystemParam, P1: SystemParam> SystemParams for (P0, P1) {
    const MUTABLE: bool = P0::MUTABLE || P1::MUTABLE;
}

pub trait System {
    fn call(&self, state: Arc<RwLock<WorldState>>);
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

impl<S: Fn(P), P: SystemParam> System for ParallelSystem<S, P> {
    fn call(&self, state: Arc<RwLock<WorldState>>) {
        (self.f)(P::fetch(state));
    }
}

impl<S: Fn(P0, P1), P0: SystemParam, P1: SystemParam> System for ParallelSystem<S, (P0, P1)> {
    fn call(&self, state: Arc<RwLock<WorldState>>) {
        (self.f)(P0::fetch(state.clone()), P1::fetch(state));
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

impl<S, P0, P1> IntoSystem<S, (P0, P1)> for S 
where
    S: Fn(P0, P1) + Send + Sync + 'static,
    P0: SystemParam + Send + Sync + 'static,
    P1: SystemParam + Send + Sync + 'static
{
    fn into_boxed(self) -> Arc<dyn System + Send + Sync> {
        if <(P0, P1)>::MUTABLE {
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

    pub async fn run_all(&self, state: &Arc<RwLock<WorldState>>) {
        let mut task_set = JoinSet::new();
        let lock = self.parallel.read();
        
        for system in &*lock {
            let state = Arc::clone(state);
            let clone = Arc::clone(system);
            task_set.spawn(async move {
                clone.call(state);
            });
        }

        // Wait for all systems to finish running
        while task_set.join_next().await.is_some() {}
    }
}