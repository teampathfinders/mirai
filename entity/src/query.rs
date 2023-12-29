use crate::component::{Component, Components};
use crate::entity::{Entity, EntityId};
use crate::world::World;
use std::marker::PhantomData;
use crate::system::SysParam;

/// A collection of [`QueryReference`].
pub trait QueryBundle {
    const EXCLUSIVE: bool;
}

/// Represents a reference to a component.
pub trait QueryReference<'w> {
    /// Indicates whether the reference is mutable or immutable.
    const MUTABLE: bool;

    fn fetch(entity: EntityId, components: &'w Components) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }
}

impl<'w, T: Component + 'static> QueryReference<'w> for &'w T {
    const MUTABLE: bool = false;
}

impl<'w, T: Component + 'static> QueryReference<'w> for &'w mut T {
    const MUTABLE: bool = true;
}

impl<'w, Q> QueryBundle for Q
where
    Q: QueryReference<'w>
{
    const EXCLUSIVE: bool = Q::MUTABLE;
}

impl<'w1, 'w2, Q1, Q2> QueryBundle for (Q1, Q2)
where
    Q1: QueryReference<'w1>, Q2: QueryReference<'w2>
{
    const EXCLUSIVE: bool = Q1::MUTABLE || Q2::MUTABLE;
}

impl<'w1, 'w2, 'w3, Q1, Q2, Q3> QueryBundle for (Q1, Q2, Q3)
where
    Q1: QueryReference<'w1>, Q2: QueryReference<'w2>, Q3: QueryReference<'w3>
{
    const EXCLUSIVE: bool = Q1::MUTABLE || Q2::MUTABLE || Q3::MUTABLE;
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

pub struct Query<'w, Q: QueryBundle, F: FilterBundle = ()> {
    components: &'w Components,
    _marker: PhantomData<(Q, F)>,
}

impl<'w, Q: QueryBundle, F: FilterBundle> Query<'w, Q, F> {
    pub(crate) fn new(components: &'w Components) -> Self {
        Self { components, _marker: PhantomData }
    }
}

impl<'w, Q: QueryBundle, F: FilterBundle> SysParam<'w> for Query<'w, Q, F> {
    const MUTABLE: bool = Q::EXCLUSIVE;

    fn fetch(components: &'w Components) -> Self {
        Query::new(components)
    }
}

impl<'w, T: QueryBundle, F: FilterBundle> Iterator for Query<'w, T, F> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
