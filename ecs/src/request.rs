use std::marker::PhantomData;

use crate::{Component, filter::ReqFilter};

pub enum AccessVariant {
    Exclusive,
    Shared
}

pub trait ReqComponents {
    const VARIANT: AccessVariant;
}

impl<T> ReqComponents for T where T: Component {
    const VARIANT: AccessVariant = AccessVariant::Shared;
}

pub struct Req<C, F = ()>
where
    C: ReqComponents,
    F: ReqFilter,
{
    _marker: PhantomData<(C, F)>
}

impl<C, F> IntoIterator for Req<C, F>
where
    C: ReqComponents,
    F: ReqFilter,
{
    type IntoIter = ReqIter<C, F>;
    type Item = C;

    fn into_iter(self) -> Self::IntoIter {
        ReqIter {
            _marker: PhantomData
        }
    }   
}

pub struct ReqIter<C, F>
where
    C: ReqComponents,
    F: ReqFilter
{
    _marker: PhantomData<(C, F)>
}

impl<C, F> Iterator for ReqIter<C, F> 
where 
    C: ReqComponents,
    F: ReqFilter
{
    type Item = C;

    fn next(&mut self) -> Option<Self::Item> {
        todo!();
    }   
}