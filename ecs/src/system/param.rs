use std::sync::Arc;

use parking_lot::RwLock;

use crate::{world::WorldState, request::{Request, Filters, Requestable, RequestCheck}};

pub trait Param {
    const READONLY: bool;

    fn fetch(state: Arc<RwLock<WorldState>>) -> Self;
}   

impl<'a, S: Requestable<'a>, F: Filters> Param for Request<'a, S, F> {
    const READONLY: bool = S::READONLY;

    fn fetch(state: Arc<RwLock<WorldState>>) -> Self {
        Request::new(state)
    }
}

pub trait ParamSet {
    const READONLY: bool;
}

impl<P> ParamSet for P 
where
    P: Param
{
    const READONLY: bool = P::READONLY;
}