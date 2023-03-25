use std::marker::PhantomData;

use crate::{request::{Req, WorldReference}, filter::FilterCollection, component::{Spawnable}, World, Requestable};

pub trait SysParam<'w>: Sized {
    const SHAREABLE: bool;

    fn fetch(_world: &'w World<'w>) -> Self {
        unreachable!("This system parameter does not support immutable world access");
    }

    fn fetch_mut(_world: &'w mut World<'w>) -> Self {
        unreachable!("This system parameter does not support mutable world access");
    }
}

impl<'w, C, F> SysParam<'w> for Req<'w, C, F>
where
    C: Requestable,
    F: FilterCollection,
{
    const SHAREABLE: bool = C::SHAREABLE;

    fn fetch(world: &'w World<'w>) -> Self {
        Req {
            world: WorldReference::Shared(world),
            _marker: PhantomData
        }
    }

    fn fetch_mut(world: &'w mut World<'w>) -> Self {
        Req {
            world: WorldReference::Exclusive(world),
            _marker: PhantomData
        }
    }
}

pub trait SysParamList {
    
}

impl<'w, S> SysParamList for S where S: SysParam<'w> {
    
}

pub trait System<'w> {
    fn run(&self, world: &'w World<'w>) {
        unreachable!("This system does not support immutable world access");
    }

    fn run_mut(&self, world: &'w mut World<'w>) {
        unreachable!("This system does not support mutable world access");
    }
}

pub trait SharedSystem<'w, Params> {
    fn run(&self, world: &'w World<'w>);
}

impl<'w, F, P> SharedSystem<'w, P> for F 
where
    F: Fn(P),
    P: SysParam<'w>
{
    fn run(&self, world: &'w World<'w>) {
        self(P::fetch(world));
    }
}

impl<'w, F, P0, P1> SharedSystem<'w, (P0, P1)> for F
where
    F: Fn(P0, P1),
    P0: SysParam<'w>, P1: SysParam<'w>,
{
    fn run(&self, world: &'w World<'w>) {
        self(P0::fetch(world), P1::fetch(world))
    }
}

pub struct SharedContainer<'w, F, P> 
where
    F: SharedSystem<'w, P>
{
    shared: F,
    _marker: PhantomData<&'w P>
}

impl<'w, F, P> SharedContainer<'w, F, P>
where
    F: SharedSystem<'w, P>,
{
    pub fn new(shared: F) -> Self {
        Self {
            shared,
            _marker: PhantomData
        }
    }
}

impl<'w, F, P> System<'w> for SharedContainer<'w, F, P>
where
    F: SharedSystem<'w, P>,
{
    fn run(&self, world: &'w World<'w>) {
        self.shared.run(world);
    }

    fn run_mut(&self, world: &'w mut World<'w>) {
        self.shared.run(world);
    }
}

pub trait IntoSystem<'w, Sys, Params> 
where
    Sys: SharedSystem<'w, Params>
{
    fn into_system(self) -> SharedContainer<'w, Sys, Params>;
}

impl<'w, Sys, Params> IntoSystem<'w, Sys, Params> for Sys
where
    Sys: Fn(Params) + 'static,
    Params: SysParam<'w> + 'static,
{
    fn into_system(self) -> SharedContainer<'w, Sys, Params> {
        if Params::SHAREABLE {
            SharedContainer::new(self)
        } else {
            todo!();
        }

        // let container = ::new(self);
        // Box::new(container)
    }
}

pub struct Executor<'w> {
    systems: Vec<Box<dyn System<'w>>>
}

impl<'w> Executor<'w> {
    pub fn new() -> Executor<'w> {
        Executor::default()
    }

    pub fn schedule(&mut self, system: Box<dyn System<'w>>) {
        self.systems.push(system);
    }

    pub fn execute(&self, world: &'w World<'w>) {
        for sys in &self.systems {
            sys.run(world);
        }
    }
}

impl<'w> Default for Executor<'w> {
    fn default() -> Executor<'w> {
        Executor {
            systems: Vec::new()
        }
    }
}