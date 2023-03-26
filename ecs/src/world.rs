use std::{sync::Arc, marker::PhantomData};

pub struct EntityId(usize);

pub struct Entity<'a> {
    world: Arc<World<'a>>,
    id: EntityId,
    _marker: PhantomData<&'a ()>
}

pub trait Spawnable {
    
}

pub trait RefComponent {
    const MUTABLE: bool;    
}

impl<T: Component> RefComponent for &T {
    const MUTABLE: bool = false;
}

impl<T: Component> RefComponent for &mut T {
    const MUTABLE: bool = true;
}

pub trait Filters {

}

impl Filters for () {}
impl<T: Component> Filters for With<T> {}
impl<T: Component> Filters for Without<T> {}

pub struct With<T: Component> {
    _marker: PhantomData<T>
}

pub struct Without<T: Component> {
    _marker: PhantomData<T>
}

pub trait Requestable {
    const MUTABLE: bool;
}

impl<T: RefComponent> Requestable for T {
    const MUTABLE: bool = T::MUTABLE;
}

impl<T0: RefComponent, T1: RefComponent> Requestable for (T0, T1) {
    const MUTABLE: bool = T0::MUTABLE || T1::MUTABLE;
}

pub trait Component {

}

impl<T: Component> Spawnable for T {}
impl<T0: Component, T1: Component> Spawnable for (T0, T1) {}

pub trait SystemParam {
    const MUTABLE: bool;

    type Item<'q>;

    fn fetch<'q>(state: &'q WorldState) -> Self::Item<'q>;
}   

impl<S: Requestable, F: Filters> SystemParam for Req<S, F> {
    const MUTABLE: bool = S::MUTABLE;

    type Item<'q> = Req<S, F>;

    fn fetch<'q>(state: &'q WorldState) -> Self::Item<'q> {

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
    fn call(&self);
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
    fn call(&self) {
        // (self.f)();
    }
}

impl<S: Fn(P0, P1), P0: SystemParam, P1: SystemParam> System for ParallelSystem<S, (P0, P1)> {
    fn call(&self) {

    }
}

pub struct Req<S, F = ()>
where
    S: Requestable,
    F: Filters,
{
    _marker: PhantomData<(S, F)>
}

impl<'q, S, F> IntoIterator for &'q Req<S, F> 
where
    S: Requestable,
    F: Filters
{
    type IntoIter = ReqIter<'q, S, F>;
    type Item = S;

    fn into_iter(self) -> Self::IntoIter {
        ReqIter { req: self }
    }
}

pub struct ReqIter<'q, S: Requestable, F: Filters> {
    req: &'q Req<S, F>
}

impl<'q, S: Requestable, F: Filters> Iterator for ReqIter<'q, S, F> {
    type Item = S;

    fn next(&mut self) -> Option<S> {
        todo!();
    }
}

pub trait IntoSystem<'a, S, Params> {
    fn into_boxed(self) -> Box<dyn System + 'a>;
}

impl<'a, S, P> IntoSystem<'a, S, P> for S 
where
    S: Fn(P) + 'a,
    P: SystemParam + 'a
{
    fn into_boxed(self) -> Box<dyn System + 'a> {
        if P::MUTABLE {
            todo!();
        } else {
            Box::new(ParallelSystem::new(self))
        }
    }
}

impl<'a, S, P0, P1> IntoSystem<'a, S, (P0, P1)> for S 
where
    S: Fn(P0, P1) + 'a,
    P0: SystemParam + 'a,
    P1: SystemParam + 'a
{
    fn into_boxed(self) -> Box<dyn System + 'a> {
        if <(P0, P1)>::MUTABLE {
            todo!();
        } else {
            Box::new(ParallelSystem::new(self))
        }
    }
}

pub struct WorldState {
    entities: Entities,
    components: Components
}

#[derive(Default)]
pub struct World<'a> {
    state: RwLock<WorldState>,
    _marker: PhantomData<&'a ()>
}

impl<'a> World<'a> {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn spawn(&self, spawnable: impl Spawnable) -> Entity {
        todo!();
    }

    pub fn system<S, P: SystemParams>(&self, systems: impl IntoSystem<'a, S, P>) {

    }
}