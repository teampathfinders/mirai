use std::marker::PhantomData;

use crate::{component::Component, filter::ReqFilter};

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

impl<'a, C, F> IntoIterator for &'a Req<C, F>
where
    C: ReqComponents,
    F: ReqFilter,
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
    C: ReqComponents,
    F: ReqFilter
{
    req: &'a Req<C, F>
}

impl<C, F> Iterator for ReqIter<'_, C, F> 
where 
    C: ReqComponents,
    F: ReqFilter
{
    type Item = C;

    fn next(&mut self) -> Option<Self::Item> {
        todo!();
    }   
}