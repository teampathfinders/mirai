use std::marker::PhantomData;

use crate::{request::{Req, ReqComponents}, filter::ReqFilter};

pub trait SysParam {
    
}

impl<C, F> SysParam for Req<C, F>
where
    C: ReqComponents,
    F: ReqFilter,
{
    
}

pub trait SysParamList {
    
}

impl<S> SysParamList for S where S: SysParam {
    
}

pub trait System {
    
}

pub struct SystemContainer<F, Params>
where
    F: IntoSystem<Params>,
    Params: SysParamList
{
    sys: F,
    _marker: PhantomData<Params>
}

impl<F, Params> System for SystemContainer<F, Params> 
where
    F: IntoSystem<Params>,
    Params: SysParamList
{

}

impl<F, P1> From<F> for SystemContainer<F, P1> 
where
    F: Fn(P1) + 'static,
    P1: SysParam + 'static 
{
    fn from(sys: F) -> SystemContainer<F, P1> {
        SystemContainer {
            sys, _marker: PhantomData
        }
    }
}

pub trait IntoSystem<Params> {
    fn into_system(self) -> Box<dyn System>;
}

impl<Sys, Params> IntoSystem<Params> for Sys
where
    Sys: Fn(Params) + 'static,
    Params: SysParam + 'static,
{
    fn into_system(self) -> Box<dyn System> {
        let container = SystemContainer::from(self);
        Box::new(container)
    }
}