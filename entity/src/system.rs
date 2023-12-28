use std::fmt::Debug;
use std::marker::PhantomData;
use crate::query::{FilterBundle, Query, QueryBundle};
use crate::world::World;

pub trait Sys<'a> {
    fn call(&self, world: &'a World);
}

pub struct SysContainer<'a, Params, F: NakedSys<'a, Params>> {
    pub(crate) system: F,
    pub(crate) _marker: PhantomData<&'a Params>
}

impl<'a, F> Sys<'a> for SysContainer<'a, (), F>
where
    F: NakedSys<'a, ()>
{
    fn call(&self, world: &'a World) {
        self.system.call(world);
    }
}

impl<'a, P, F> Sys<'a> for SysContainer<'a, P, F>
where
    F: NakedSys<'a, P>,
    P: SysParam<'a>
{
    fn call(&self, world: &'a World) {
        self.system.call(world);
    }
}

impl<'a, P1, P2, F> Sys<'a> for SysContainer<'a, (P1, P2), F>
where
    F: NakedSys<'a, (P1, P2)>,
    P1: SysParam<'a>, P2: SysParam<'a>
{
    fn call(&self, world: &'a World) {
        self.system.call(world);
    }
}

impl<'a, P1, P2, P3, F> Sys<'a> for SysContainer<'a, (P1, P2, P3), F>
where
    F: NakedSys<'a, (P1, P2, P3)>,
    P1: SysParam<'a>, P2: SysParam<'a>, P3: SysParam<'a>
{
    fn call(&self, world: &'a World) {
        self.system.call(world);
    }
}

pub trait SysParam<'a>: Sized {
    const EXCLUSIVE: bool;

    fn fetch(world: &'a World) -> Self {
        panic!("{} does not support immutable fetching", std::any::type_name::<Self>());
    }

    fn fetch_mut(world: &'a mut World) -> Self {
        panic!("{} does not support mutable fetching", std::any::type_name::<Self>());
    }
}

impl<'a, Q: QueryBundle, F: FilterBundle> SysParam<'a> for Query<'a, Q, F> {
    const EXCLUSIVE: bool = Q::EXCLUSIVE;
}

pub trait SysParamBundle<'a>: Sized {
    const EXCLUSIVE: bool;
}

impl<'a> SysParamBundle<'a> for () {
    const EXCLUSIVE: bool = false;
}

impl<'a, P: SysParam<'a>> SysParamBundle<'a> for P {
    const EXCLUSIVE: bool = P::EXCLUSIVE;
}

impl<'a, P1, P2> SysParamBundle<'a> for (P1, P2)
    where P1: SysParam<'a>, P2: SysParam<'a>
{
    const EXCLUSIVE: bool = P1::EXCLUSIVE || P2::EXCLUSIVE;
}

impl<'a, P1, P2, P3> SysParamBundle<'a> for (P1, P2, P3)
    where P1: SysParam<'a>, P2: SysParam<'a>, P3: SysParam<'a>
{
    const EXCLUSIVE: bool = P1::EXCLUSIVE || P2::EXCLUSIVE || P3::EXCLUSIVE;
}

pub trait NakedSys<'a, Params>: Sized {
    fn into_generic(self) -> SysContainer<'a, Params, Self> {
        SysContainer { system: self, _marker: PhantomData }
    }

    fn call(&self, world: &'a World);
}

impl<'a, F> NakedSys<'a, ()> for F where F: Fn() {
    fn call(&self, _world: &'a World) {
        self();
    }
}

impl<'a, F, P> NakedSys<'a, P> for F
where
    F: Fn(P), P: SysParam<'a>,
{
    fn call(&self, world: &'a World) {
        let p = P::fetch(world);
        self(p);
    }
}

impl<'a, F, P1, P2> NakedSys<'a, (P1, P2)> for F
where
    F: Fn(P1, P2), P1: SysParam<'a>, P2: SysParam<'a>
{
    fn call(&self, world: &'a World) {
        let p1 = P1::fetch(world);
        let p2 = P2::fetch(world);

        self(p1, p2);
    }
}

pub struct Systems {
    // pub(crate) exclusive: Vec<Box<dyn ExclusiveSystem>>,
    // pub(crate) parallel: Vec<Box<dyn ParallelSystem>>
}

impl Systems {
    pub fn new() -> Self {
        Systems {
            // exclusive: Vec::new(),
            // parallel: Vec::new()
        }
    }

    pub fn insert<'a, P, S>(&mut self, system: S)
    where
        P: SysParamBundle<'a> + 'static,
        S: NakedSys<'a, P> + 'static,
        SysContainer<'a, P, S>: Sys<'a>
    {
        println!("is exclusive: {}", P::EXCLUSIVE);
        // todo!()
    }
}
