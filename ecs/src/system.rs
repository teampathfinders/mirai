use std::marker::PhantomData;

use crate::{request::{Req}, filter::FilterCollection, component::{Spawnable, Requestable}, World};

pub trait SysParam {
    fn fetch(w: &World) -> Self;
}

impl<C, F> SysParam for Req<C, F>
where
    C: Requestable,
    F: FilterCollection,
{
    fn fetch(_w: &World) -> Self {
        todo!();
    }
}

pub trait SysParamList {
    
}

impl<S> SysParamList for S where S: SysParam {
    
}

pub trait System {
    fn run(&self, world: &World) {
        unreachable!("This system does not support immutable world access");
    }

    fn run_mut(&self, world: &mut World) {
        unreachable!("This system does not support mutable world access");
    }
}

pub trait SharedSystem<Params> {
    fn run(&self, world: &World);
}

impl<F, P> SharedSystem<P> for F 
where
    F: Fn(P),
    P: SysParam
{
    fn run(&self, world: &World) {
        self(P::fetch(world));
    }
}

impl<F, P0, P1> SharedSystem<(P0, P1)> for F
where
    F: Fn(P0, P1),
    P0: SysParam, P1: SysParam,
{
    fn run(&self, world: &World) {
        self(P0::fetch(world), P1::fetch(world))
    }
}

pub struct SystemContainer<F, P>
where
    F: SharedSystem<P>,
    P: SysParamList,
{
    sys: F,
    _marker: PhantomData<P>
}

impl<F, P> SystemContainer<F, P>
where
    F: SharedSystem<P>,
    P: SysParamList,
{
    pub fn new(sys: F) -> Self {
        Self {
            sys,
            _marker: PhantomData
        }
    }
}

impl<F, P> System for SystemContainer<F, P>
where
    F: SharedSystem<P>,
    P: SysParamList,
{
    fn run(&self, world: &World) {
        self.sys.run(world);
    }
}

pub trait IntoSystem<Params> {
    fn into_system(self) -> Box<dyn System>;
}

impl<Sys, Params> IntoSystem<Params> for Sys
where
    Sys: Fn(Params) + 'static,
    Params: SysParam + 'static,
{
    fn into_system(self) -> Box<dyn System> {
        let container = SystemContainer::new(self);
        Box::new(container)
    }
}

pub struct Executor {
    systems: Vec<Box<dyn System>>
}

impl Executor {
    pub fn new() -> Executor {
        Executor::default()
    }

    pub fn schedule(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }

    pub fn execute(&self, world: &World) {
        for sys in &self.systems {
            sys.run(world);
        }
    }
}

impl Default for Executor {
    fn default() -> Executor {
        Executor {
            systems: Vec::new()
        }
    }
}