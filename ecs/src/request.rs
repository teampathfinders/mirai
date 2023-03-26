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
    entities: &'r [bool],
    store: StoreReference<'r>,
    _marker: PhantomData<(R, F)>
}

impl<'r, R, F> Req<'r, R, F> 
where
    R: Requestable,
    F: FilterCollection
{
    pub fn new(entities: &'r [bool], store: &'r ComponentStore) -> Self {
        Self {
            entities,
            store: StoreReference::Immutable(store),
            _marker: PhantomData
        }
    }
}

impl<'r, R, F> IntoIterator for &'r Req<'r, R, F>
where
    R: Requestable + 'r,
    F: FilterCollection + 'r,
{
    type IntoIter = ReqIter<'r, R, F>;
    type Item = R;

    fn into_iter(self) -> Self::IntoIter {
        ReqIter::from(self)
    }
}

pub struct ReqIter<'r, R, F> 
where
    R: Requestable,
    F: FilterCollection
{
    req: &'r Req<'r, R, F>,
}

impl<'r, R, F> Iterator for ReqIter<'r, R, F> 
where
    R: Requestable,
    F: FilterCollection
{
    type Item = R;

    fn next(&mut self) -> Option<R> {
        todo!();
    }
}

impl<'r, R, F> From<&'r Req<'r, R, F>> for ReqIter<'r, R, F> 
where
    R: Requestable,
    F: FilterCollection
{
    fn from(req: &'r Req<'r, R, F>) -> Self {
        Self {
            req
        }
    }
}