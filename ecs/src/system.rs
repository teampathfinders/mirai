use std::marker::PhantomData;

use crate::{request::{Req, ReqComponents}, filter::ReqFilter};

pub trait SysParam {

}

impl<C, F> SysParam for Req<C, F>
where
    C: ReqComponents,
    F: ReqFilter,
{}

pub trait System {
    fn print(&self) {
        println!("{}", std::any::type_name::<Self>());
    }
}

pub struct SystemContainer<F, P> {
    sys: F,
    _marker: PhantomData<P>
}

impl<F, P> System for SystemContainer<F, P> {
    
}

impl<F, P1> From<F> for SystemContainer<F, P1> 
where
    F: Fn(P1),
    P1: SysParam
{
    fn from(sys: F) -> SystemContainer<F, P1> {
        SystemContainer {
            sys, _marker: PhantomData
        }
    }
}

pub trait IntoSystem<F, P1> {
    fn into_system(self) -> Box<dyn System>;
}

impl<F, P1> IntoSystem<F, P1> for F
where
    F: Fn(P1) + 'static,
    P1: SysParam + 'static,
{
    fn into_system(self) -> Box<dyn System> {
        let container = SystemContainer::<F, P1>::from(self);
        Box::new(container)
    }
}