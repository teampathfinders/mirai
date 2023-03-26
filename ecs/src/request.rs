use std::marker::PhantomData;

use crate::{component::{Component, Spawnable, RefComponent, ComponentStore}, filter::FilterCollection, World};

pub trait Requestable: Sized {
    const PARALLEL: bool;

    fn fetch(entity: usize, store: &StoreReference) -> Option<Self>;
}

impl<T> Requestable for T
where
    T: RefComponent,
{
    const PARALLEL: bool = T::SHAREABLE;

    fn fetch(entity: usize, store: &StoreReference) -> Option<Self> {
        debug_assert_eq!(Self::PARALLEL, store.is_immutable());
        todo!();
    }
}

impl<T0, T1> Requestable for (T0, T1)
where
    T0: RefComponent,
    T1: RefComponent,
{
    const PARALLEL: bool = T0::SHAREABLE && T1::SHAREABLE;

    fn fetch(entity: usize, store: &StoreReference) -> Option<Self> {
        todo!();
    }
}

pub enum StoreReference<'r> {
    Immutable(&'r ComponentStore)
}

impl StoreReference<'_> {
    pub fn is_immutable(&self) -> bool {
        matches!(self, Self::Immutable(_))
    }
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
    index: usize,
    entities: &'r [bool],
    store: &'r StoreReference<'r>,
    _marker: PhantomData<(R, F)>
}

impl<'r, R, F> Iterator for ReqIter<'r, R, F> 
where
    R: Requestable,
    F: FilterCollection
{
    type Item = R;

    fn next(&mut self) -> Option<R> {
        let (nth, requested) = self.entities  
            .iter()
            .enumerate()
            .find_map(|(i, v)| if !*v {
                None
            } else {
                let r = R::fetch(self.index + i, self.store)?;
                Some((i, r))
            })?;

        self.index += nth;
        Some(requested)
    }
}

impl<'r, R, F> From<&'r Req<'r, R, F>> for ReqIter<'r, R, F> 
where
    R: Requestable,
    F: FilterCollection
{
    fn from(req: &'r Req<'r, R, F>) -> Self {
        Self {
            index: 0,
            entities: req.entities,
            store: &req.store,
            _marker: PhantomData
        }
    }
}