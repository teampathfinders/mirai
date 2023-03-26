use std::marker::PhantomData;

use crate::{component::{Component, Spawnable, RefComponent, ComponentStore}, filter::FilterCollection, World};

pub trait Requestable {
    const PARALLEL: bool;
}

impl<T> Requestable for T
where
    T: RefComponent,
{
    const PARALLEL: bool = T::SHAREABLE;
}

impl<T0, T1> Requestable for (T0, T1)
where
    T0: RefComponent,
    T1: RefComponent,
{
    const PARALLEL: bool = T0::SHAREABLE && T1::SHAREABLE;
}

enum StoreReference<'r> {
    Immutable(&'r ComponentStore)
}

pub struct Req<'r, R, F = ()> 
where
    R: Requestable,
    F: FilterCollection
{
    store: StoreReference<'r>,
    _marker: PhantomData<(R, F)>
}

impl<'r, R, F> From<&'r ComponentStore> for Req<'r, R, F> 
where
    R: Requestable,
    F: FilterCollection
{
    fn from(value: &'r ComponentStore) -> Self {
        Self {
            store: StoreReference::Immutable(value),
            _marker: PhantomData
        }
    }
}