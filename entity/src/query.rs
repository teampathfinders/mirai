use crate::component::{Component, Components};
use crate::entity::{Entity, EntityId};
use crate::world::World;
use std::marker::PhantomData;
use crate::system::SysParam;

pub trait QueryBundle {
    const EXCLUSIVE: bool;
}

pub trait QueryReference<'w> {
    const EXCLUSIVE: bool;

    fn fetch(entity: Entity, components: &'w Components) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }
}

impl<'w, T: Component + 'static> QueryReference<'w> for &'w T {
    const EXCLUSIVE: bool = false;
}

impl<'w, T: Component + 'static> QueryReference<'w> for &'w mut T {
    const EXCLUSIVE: bool = true;
}

impl<'w, Q> QueryBundle for Q
where
    Q: QueryReference<'w>
{
    const EXCLUSIVE: bool = Q::EXCLUSIVE;
}

impl<'w1, 'w2, Q1, Q2> QueryBundle for (Q1, Q2)
where
    Q1: QueryReference<'w1>, Q2: QueryReference<'w2>
{
    const EXCLUSIVE: bool = Q1::EXCLUSIVE || Q2::EXCLUSIVE;
}

impl<'w1, 'w2, 'w3, Q1, Q2, Q3> QueryBundle for (Q1, Q2, Q3)
where
    Q1: QueryReference<'w1>, Q2: QueryReference<'w2>, Q3: QueryReference<'w3>
{
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

impl<'w, Q: QueryBundle, F: FilterBundle> SysParam<'w> for Query<'w, Q, F> {
    const EXCLUSIVE: bool = Q::EXCLUSIVE;
}

impl<'w, T: QueryBundle, F: FilterBundle> Iterator for Query<'w, T, F> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
