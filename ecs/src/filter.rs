use std::marker::PhantomData;

use crate::component::Component;

pub struct With<C> {
    _marker: PhantomData<C>
}

pub struct Without<C> {
    _marker: PhantomData<C>
}

pub trait ReqFilter {
    fn filter() -> bool;
}

impl ReqFilter for () {
    fn filter() -> bool {
        // Don't filter anything 
        true
    }
}

impl<F> ReqFilter for With<F> where F: Component {
    fn filter() -> bool {
        println!("Filtering with {}", std::any::type_name::<Self>());
        true
    }
}

impl<F> ReqFilter for Without<F> where F: Component {
    fn filter() -> bool {
        true
    }
}