use std::marker::PhantomData;

use crate::{component::{Component, Spawnable, RefComponent}, filter::FilterCollection, World};

pub trait Requestable {
    const SHAREABLE: bool;
}

impl<T> Requestable for T
where
    T: RefComponent,
{
    const SHAREABLE: bool = T::SHAREABLE;
}

impl<T0, T1> Requestable for (T0, T1)
where
    T0: RefComponent,
    T1: RefComponent,
{
    const SHAREABLE: bool = T0::SHAREABLE && T1::SHAREABLE;
}

pub enum WorldReference<'w> {
    Shared(&'w World<'w>),
    Exclusive(&'w mut World<'w>)
}

pub struct Req<'w, C, F = ()>
where
    C: Requestable,
    F: FilterCollection,
{
    pub(crate) world: WorldReference<'w>,
    pub(crate) _marker: PhantomData<(C, F)>
}

impl<'r, 'w, C, F> IntoIterator for &'r Req<'w, C, F>
where
    C: Requestable,
    F: FilterCollection,
{
    type IntoIter = ReqIter<'r, 'w, C, F>;
    type Item = C;

    fn into_iter(self) -> Self::IntoIter {
        ReqIter {
            req: self
        }
    }   
}

pub struct ReqIter<'r, 'w, C, F>
where
    C: Requestable,
    F: FilterCollection
{
    req: &'r Req<'w, C, F>
}

impl<'r, 'w, C, F> Iterator for ReqIter<'r, 'w, C, F> 
where 
    C: Requestable,
    F: FilterCollection
{
    type Item = C;

    fn next(&mut self) -> Option<Self::Item> {
        todo!();
    }   
}