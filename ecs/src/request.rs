use std::marker::PhantomData;

use crate::Component;

pub enum AccessVariant {
    Exclusive,
    Shared
}

pub struct With<C> {
    _marker: PhantomData<C>
}

pub struct Without<C> {
    _marker: PhantomData<C>
}

pub struct Request<C, F = ()>
where
    C: RequestComponents,
    F: RequestFilters,
{
    _marker: PhantomData<(C, F)>
}

pub trait RequestComponents {
    const VARIANT: AccessVariant;
}

impl<T> RequestComponents for T where T: Component {
    const VARIANT: AccessVariant = AccessVariant::Shared;
}

pub trait RequestFilters {

}

impl RequestFilters for () {}

impl<F> RequestFilters for With<F> where F: Component {}
impl<F> RequestFilters for Without<F> where F: Component {}