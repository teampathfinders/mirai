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

pub trait SysParam<'w>: Sized {
    const EXCLUSIVE: bool;

    fn fetch(world: &'w World) -> Self {
        panic!("{} does not support immutable fetching", std::any::type_name::<Self>());
    }

    fn fetch_mut(world: &'w mut World) -> Self {
        panic!("{} does not support mutable fetching", std::any::type_name::<Self>());
    }
}

pub trait SysParamBundle: Sized {
    const EXCLUSIVE: bool;
}

impl SysParamBundle for () {
    const EXCLUSIVE: bool = false;
}

impl<'w, P: SysParam<'w>> SysParamBundle for P {
    const EXCLUSIVE: bool = P::EXCLUSIVE;
}

impl<'w1, 'w2, P1, P2> SysParamBundle for (P1, P2)
    where P1: SysParam<'w1>, P2: SysParam<'w2>
{
    const EXCLUSIVE: bool = P1::EXCLUSIVE || P2::EXCLUSIVE;
}

impl<'w1, 'w2, 'w3, P1, P2, P3> SysParamBundle for (P1, P2, P3)
    where P1: SysParam<'w1>, P2: SysParam<'w2>, P3: SysParam<'w3>
{
    const EXCLUSIVE: bool = P1::EXCLUSIVE || P2::EXCLUSIVE || P3::EXCLUSIVE;
}

pub trait NakedSys<'w, P>: Sized {
    fn into_container(self) -> SysContainer<'w, P, Self> {
        SysContainer { system: self, _marker: PhantomData }
    }

    fn call(&self, world: &'w World);
}

impl<'w, F> NakedSys<'w, ()> for F where F: Fn() {
    fn call(&self, _world: &'w World) {
        self();
    }
}

impl<'w, F, P> NakedSys<'w, P> for F
where
    F: Fn(P), P: SysParam<'w>,
{
    fn call(&self, world: &'w World) {
        let p = P::fetch(world);
        self(p);
    }
}

impl<'w, F, P1, P2> NakedSys<'w, (P1, P2)> for F
where
    F: Fn(P1, P2), P1: SysParam<'w>, P2: SysParam<'w>
{
    fn call(&self, world: &'w World) {
        let p1 = P1::fetch(world);
        let p2 = P2::fetch(world);

        self(p1, p2);
    }
}

impl<'w, F, P1, P2, P3> NakedSys<'w, (P1, P2, P3)> for F
where
    F: Fn(P1, P2, P3), P1: SysParam<'w>, P2: SysParam<'w>, P3: SysParam<'w>
{
    fn call(&self, world: &'w World) {
        let p1 = P1::fetch(world);
        let p2 = P2::fetch(world);
        let p3 = P3::fetch(world);

        self(p1, p2, p3);
    }
}

pub struct Systems {
    pub(crate) exclusive: Vec<Box<dyn for<'a> Sys<'a>>>,
    pub(crate) parallel: Vec<Box<dyn for<'a> Sys<'a>>>
}

impl Systems {
    pub fn new() -> Self {
        Systems {
            exclusive: Vec::new(),
            parallel: Vec::new()
        }
    }

    pub fn insert<'w, P, S>(&mut self, system: S)
    where
        P: SysParamBundle + 'static,
        S: NakedSys<'w, P> + 'static,
        SysContainer<'w, P, S>: Sys<'w>
    {
        println!("is exclusive: {}", P::EXCLUSIVE);
        // todo!()
    }
}
