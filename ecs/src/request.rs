use std::{sync::{Arc}, marker::PhantomData, pin::Pin};

use parking_lot::{RwLock, RwLockReadGuard};

use crate::{private, component::{Component, Components}, entity::{Entity, EntityId, Entities}, world::WorldState};

pub trait Filters {
    fn filter(entity: usize, state: &WorldState) -> bool;
}

impl Filters for () {
    fn filter(_entity: usize, _state: &WorldState) -> bool {
        true
    }
}

impl<T: Component + 'static> Filters for With<T> {
    fn filter(entity: usize, state: &WorldState) -> bool {
        state.entity_has::<T>(entity)
    }
}

impl<T: Component + 'static> Filters for Without<T> {
    fn filter(entity: usize, state: &WorldState) -> bool {
        !state.entity_has::<T>(entity)
    }
}

pub struct With<T: Component> {
    _marker: PhantomData<T>
}

pub struct Without<T: Component> {
    _marker: PhantomData<T>
}

/// # Safety
/// 
/// [`Request`] relies on this trait correctly being implemented.
/// The [`IS_ENTITY`](Requestable::IS_ENTITY) associated constant *must* only be true for [`Entity`].
pub unsafe trait Requestable: Sized + private::Sealed {
    type Fetch<'state>;

    /// This is here because ReqIter logic requires checking whether the requestable is an entity.
    /// It cannot be done using TypeId as that requires a static lifetime, which we do not have.
    const IS_ENTITY: bool;
    const MUTABLE: bool;

    fn is_viable(entity: usize, components: &Components) -> bool;
}

impl<T> private::Sealed for &T where T: Component {}

unsafe impl<T> Requestable for &T 
where
    T: Component + 'static
{
    type Fetch<'state> = &'state T;

    const IS_ENTITY: bool = false;
    const MUTABLE: bool = false;

    fn is_viable(entity: usize, components: &Components) -> bool {
        components.entity_has::<T>(entity)
    }
}

pub struct Request<'state, S, F = ()>
where
    S: Requestable,
    F: Filters,
{   
    entities: &'state Entities,
    components: &'state Components,
    _marker: PhantomData<(S, F)>
}

impl<'state, S, F> Request<'state, S, F> 
where
    S: Requestable,
    F: Filters
{
    pub fn new(state: &'state WorldState) -> Self {
        Self {
            entities: &state.entities,
            components: &state.components,
            _marker: PhantomData
        }
    }
}

impl<'state, S, F> IntoIterator for &Request<'state, S, F> 
where
    S: Requestable + 'state,
    F: Filters + 'state
{
    type IntoIter = RequestIter<'state, S, F>;
    type Item = S::Fetch<'state>;

    fn into_iter(self) -> Self::IntoIter {
        RequestIter {
            last_entity: 0,
            entities: self.entities,
            components: self.components,
            _marker: PhantomData
        }
    }
}

pub struct RequestIter<'state, S, F> 
where
    S: Requestable,
    F: Filters
{
    last_entity: usize,
    entities: &'state Entities,
    components: &'state Components,
    _marker: PhantomData<(S, F)>
}

impl<'state, S, F> Iterator for RequestIter<'state, S, F> 
where
    S: Requestable,
    F: Filters
{
    type Item = S::Fetch<'state>;

    fn next(&mut self) -> Option<Self::Item> {
        let lock = self.entities.mapping.read();

        // Find the next matching entity
        let entity = lock
            .iter()
            .enumerate()
            .find_map(|(index, entity)| {
                if *entity {
                    // Verify that entity has requested components 
                    if S::is_viable(self.last_entity + index, self.components) {
                        return None
                    }

                    None
                } else {
                    // Entity ID is not in use
                    None
                }
            })?;

        todo!();
    }
}