use crate::component::{Component, Components};
use crate::entity::{Entity, EntityId};
use crate::world::World;
use std::marker::PhantomData;

pub trait QueryBundle {
    const EXCLUSIVE: bool;
}

pub trait QueryReference {
    const EXCLUSIVE: bool;

    fn fetch(entity: Entity, components: &Components) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }
}

impl<T: Component + 'static> QueryReference for &T {
    const EXCLUSIVE: bool = false;
}

impl<T: Component + 'static> QueryReference for &mut T {
    const EXCLUSIVE: bool = true;
}

impl<Q: QueryReference> QueryBundle for Q {
    const EXCLUSIVE: bool = Q::EXCLUSIVE;
}

impl<Q1: QueryReference, Q2: QueryReference> QueryBundle for (Q1, Q2) {
    const EXCLUSIVE: bool = Q1::EXCLUSIVE || Q2::EXCLUSIVE;
}

impl<Q1: QueryReference, Q2: QueryReference, Q3: QueryReference> QueryBundle for (Q1, Q2, Q3) {
    const EXCLUSIVE: bool = Q1::EXCLUSIVE || Q2::EXCLUSIVE || Q3::EXCLUSIVE;
}

pub trait Filter {
    fn filter(entity: EntityId) -> bool;
}

pub trait FilterBundle {
    fn filter(entity: EntityId) -> bool;
}

impl<F: Filter> FilterBundle for F {
    fn filter(entity: EntityId) -> bool {
        F::filter(entity)
    }
}

impl<F1: Filter, F2: Filter> FilterBundle for (F1, F2) {
    fn filter(entity: EntityId) -> bool {
        F1::filter(entity) && F2::filter(entity)
    }
}

impl<F1: Filter, F2: Filter, F3: Filter> FilterBundle for (F1, F2, F3) {
    fn filter(entity: EntityId) -> bool {
        F1::filter(entity) && F2::filter(entity) && F3::filter(entity)
    }
}

pub struct With<T: Component + 'static> {
    _marker: PhantomData<T>,
}

impl Filter for () {
    fn filter(_entity: EntityId) -> bool {
        true
    }
}

impl<T: Component + 'static> Filter for With<T> {
    fn filter(entity: EntityId) -> bool {
        true
    }
}

pub struct Query<'w, T: QueryBundle, F: FilterBundle = ()> {
    world: &'w World,
    _marker: PhantomData<(T, F)>,
}

impl<'w, T: QueryBundle, F: FilterBundle> Iterator for Query<'w, T, F> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
