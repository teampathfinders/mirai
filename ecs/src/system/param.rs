use crate::{world::WorldState, request::{Request, Filters, Requestable}};

pub trait Param {
    const READONLY: bool;

    type Target<'state>;

    fn fetch<'state>(state: &'state WorldState) -> Self::Target<'state>;
}   

impl<S: Requestable, F: Filters> Param for Request<'_, S, F> {
    const READONLY: bool = S::READONLY;

    type Target<'state> = Request<'state, S, F>;

    fn fetch<'state>(state: &'state WorldState) -> Self::Target<'state> {
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