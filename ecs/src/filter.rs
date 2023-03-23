use std::marker::PhantomData;

use crate::Component;

pub struct With<C> {
    _marker: PhantomData<C>
}

pub struct Without<C> {
    _marker: PhantomData<C>
}

pub trait ReqFilter {
    fn filter() -> bool {
        true
    }
}

impl ReqFilter for () {}

impl<F> ReqFilter for With<F> where F: Component {}
impl<F> ReqFilter for Without<F> where F: Component {}