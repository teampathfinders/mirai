use std::marker::PhantomData;

use crate::{component::{Component, Spawnable, RefComponent}, filter::FilterCollection, World};

pub trait Requestable {
    const PARALLEL: bool;
}

impl<T> Requestable for T
where
    T: RefComponent,
{
    const PARALLEL: bool = T::SHAREABLE;
}

impl<T0, T1> Requestable for (T0, T1)
where
    T0: RefComponent,
    T1: RefComponent,
{
    const PARALLEL: bool = T0::SHAREABLE && T1::SHAREABLE;
}

enum WorldReference<'r> {
    Shared(&'r World),
    Exclusive(&'r mut World)
}

pub struct Req<'r, R, F = ()> 
where
    R: Requestable,
    F: FilterCollection
{
    world: WorldReference<'r>,
    _marker: PhantomData<(R, F)>
}

