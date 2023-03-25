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

impl<'w> SysParam<'w> for () {
    const SHAREABLE: bool = true;

    fn fetch(_world: &'w World<'w>) -> Self {}
    fn fetch_mut(_world: &'w mut World<'w>) -> Self {}
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
    const SHAREABLE: bool;
}

impl<'w, S> SysParamList for S where S: SysParam<'w> {
    const SHAREABLE: bool = <S as SysParam>::SHAREABLE;
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

pub trait IntoSystem<'w, Sys, Params> {
    fn into_boxed(self) -> Box<dyn System<'w> + 'w>;
}

impl<'w, Sys, Params> IntoSystem<'w, Sys, Params> for Sys
where
    Sys: Fn(Params) + 'w,
    Params: SysParam<'w> + 'w,
{
    fn into_boxed(self) -> Box<dyn System<'w> + 'w> {
        if Params::SHAREABLE {
            let container = SharedContainer::new(self);
            Box::new(container)
        } else {
            todo!();
        }

        // let container = ::new(self);
        // Box::new(container)
    }
}

pub struct Executor<'w> {
    shared: Vec<Box<dyn System<'w> + 'w>>
}

impl<'w> Executor<'w> {
    pub fn new() -> Executor<'w> {
        Executor::default()
    }

    pub fn add_system<Sys, Params>(&mut self, system: impl IntoSystem<'w, Sys, Params>) 
    where
        Params: SysParamList
    {
        let boxed = system.into_boxed();
        if Params::SHAREABLE {
            self.shared.push(boxed);
        } else {
            todo!();
        }
    }

    pub fn execute(&self, world: &World<'w>) {
        for sys in &self.shared {
            sys.run(world);
        }
    }
}

impl<'w> Default for Executor<'w> {
    fn default() -> Executor<'w> {
        Executor {
            shared: Vec::new()
        }
    }
}