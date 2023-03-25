use std::marker::PhantomData;

use crate::{component::{Component, Spawnable, RefComponent}, filter::FilterCollection};

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

pub struct Req<C, F = ()>
where
    C: Requestable,
    F: FilterCollection,
{
    _marker: PhantomData<(C, F)>
}

impl<'a, C, F> IntoIterator for &'a Req<C, F>
where
    C: Requestable,
    F: FilterCollection,
{
    type IntoIter = ReqIter<'a, C, F>;
    type Item = C;

    fn into_iter(self) -> Self::IntoIter {
        ReqIter {
            req: self
        }
    }   
}

pub struct ReqIter<'a, C, F>
where
    C: Requestable,
    F: FilterCollection
{
    req: &'a Req<C, F>
}

impl<C, F> Iterator for ReqIter<'_, C, F> 
where 
    C: Requestable,
    F: FilterCollection
{
    type Item = C;

    fn next(&mut self) -> Option<Self::Item> {
        todo!();
    }   
}