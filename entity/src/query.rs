use std::marker::PhantomData;
use crate::component::{Component, Components};
use crate::entity::{Entity, EntityId};

pub trait QueryBundle {
    const MUTABLE: bool;
}

pub trait QueryReference {
    const MUTABLE: bool;

    fn fetch(entity: Entity, components: &Components) -> Option<Self> where Self: Sized { None }
}

impl<T: Component + 'static> QueryReference for &T {
    const MUTABLE: bool = false;
}

impl<T: Component + 'static> QueryReference for &mut T {
    const MUTABLE: bool = true;
}

impl<Q: QueryReference> QueryBundle for Q {
    const MUTABLE: bool = Q::MUTABLE;
}
impl<Q1: QueryReference, Q2: QueryReference> QueryBundle for (Q1, Q2) {
    const MUTABLE: bool = Q1::MUTABLE || Q2::MUTABLE;
}
impl<Q1: QueryReference, Q2: QueryReference, Q3: QueryReference> QueryBundle for (Q1, Q2, Q3) {
    const MUTABLE: bool = Q1::MUTABLE || Q2::MUTABLE || Q3::MUTABLE;
}

pub trait Filter {
    fn filter(entity: EntityId) -> bool;
}

pub trait FilterBundle {

}

impl<F: Filter> FilterBundle for F {}
impl<F1: Filter, F2: Filter> FilterBundle for (F1, F2) {}
impl<F1: Filter, F2: Filter, F3: Filter> FilterBundle for (F1, F2, F3) {}

pub struct With<T: Component + 'static> {
    _marker: PhantomData<T>
}

impl Filter for () {
    fn filter(_entity: EntityId) -> bool { true }
}

impl<T: Component + 'static> Filter for With<T> {
    fn filter(entity: EntityId) -> bool {
        true
    }
}

pub struct Query<T: QueryBundle, F: FilterBundle = ()> {
    _marker: PhantomData<(T, F)>
}

