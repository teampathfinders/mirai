use std::marker::PhantomData;

use crate::{request::{Req, Requestable}, FilterCollection, World, component::ComponentStore};

pub trait SystemParam: Sized {
    const PARALLEL: bool;

    fn fetch(components: &ComponentStore) -> Self;
}

impl SystemParam for () {
    const PARALLEL: bool = true;

    fn fetch(_components: &ComponentStore) {}
}

impl<'r, R: Requestable, F: FilterCollection> SystemParam for Req<'r, R, F> {
    const PARALLEL: bool = R::PARALLEL;

    fn fetch(components: &ComponentStore) -> Self {
        Req::from(unsafe {
            &*(components as *const ComponentStore)
        })
    }
}

pub trait SystemParams {
    const PARALLEL: bool;
}

impl<P: SystemParam> SystemParams for P {
    const PARALLEL: bool = <P as SystemParam>::PARALLEL;
}

pub trait System {
    fn run(&self, store: &ComponentStore);
}

pub struct ParallelContainer<S, P: SystemParams> {
    runnable: S,
    _marker: PhantomData<P>
}

impl<S, P> ParallelContainer<S, P> 
where
    P: SystemParams
{
    pub fn new(runnable: S) -> Self {
        debug_assert!(P::PARALLEL, "Cannot create ParallelContainer from exclusive system");
        Self {
            runnable,
            _marker: PhantomData
        }
    }
}

impl<'r, S, P> System for ParallelContainer<S, P> 
where
    S: Fn(P),
    P: SystemParam
{
    fn run(&self, store: &ComponentStore) {
        (self.runnable)(P::fetch(store));
    }
}

pub trait IntoSystem<S, P> 
where
    P: SystemParams
{
    fn into_boxed(self) -> Box<dyn System>;
}

impl<'r, S: Fn(P) + 'static, P: SystemParam + 'static> IntoSystem<S, P> for S {
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
    parallel: Vec<Box<dyn System>>
}

impl Executor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_system<S, P>(&mut self, system: impl IntoSystem<S, P>) 
    where
        P: SystemParams
    {
        if P::PARALLEL {
            let boxed = system.into_boxed();
            self.parallel.push(boxed);
        } else {
            todo!();
        }
    }

    pub fn run_all(&self, store: &ComponentStore) {
        for sys in &self.parallel {
            sys.run(store);
        }
    }
}