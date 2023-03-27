use std::{sync::{Arc}, marker::PhantomData, pin::Pin, mem};

use parking_lot::{RwLock, RwLockReadGuard};

use crate::{private, component::{Component, Components}, entity::{Entity, EntityId, Entities, EntityMut}, world::WorldState};

pub trait Filters {
    fn filter(entity: usize, components: &Components) -> bool;
}

impl Filters for () {
    fn filter(_entity: usize, _components: &Components) -> bool {
        true
    }
}

impl<T: Component + 'static> Filters for With<T> {
    fn filter(entity: usize, components: &Components) -> bool {
        components.entity_has::<T>(entity)
    }
}

impl<T: Component + 'static> Filters for Without<T> {
    fn filter(entity: usize, components: &Components) -> bool {
        !components.entity_has::<T>(entity)
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
    const READONLY: bool;

    fn is_viable(entity: usize, components: &Components) -> bool;

    fn fetch<'state>(entity: usize, components: &'state Components) -> Option<Self::Fetch<'state>> {
        unimplemented!()
    }
}

impl<T> private::Sealed for &T where T: Component {}

unsafe impl<T> Requestable for &T 
where
    T: Component + 'static
{
    type Fetch<'state> = &'state T;

    const IS_ENTITY: bool = false;
    const READONLY: bool = true;

    #[inline]
    fn is_viable(entity: usize, components: &Components) -> bool {
        components.entity_has::<T>(entity)
    }

    fn fetch<'state>(entity: usize, components: &'state Components) -> Option<Self::Fetch<'state>> {
        components.get(entity)
    }
}

impl private::Sealed for Entity<'_> {}

unsafe impl Requestable for Entity<'_> {
    type Fetch<'state> = Entity<'state>;

    const IS_ENTITY: bool = true;
    const READONLY: bool = true;

    #[inline]
    fn is_viable(entity: usize, components: &Components) -> bool {
        true
    }
}

impl private::Sealed for EntityMut<'_> {}

unsafe impl Requestable for EntityMut<'_> {
    type Fetch<'state> = EntityMut<'state>;

    const IS_ENTITY: bool = true;
    const READONLY: bool = false;

    #[inline]
    fn is_viable(entity: usize, components: &Components) -> bool {
        true
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
            next_entity: 0,
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
    next_entity: usize,
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
        let mapping = &self.entities.mapping.read()[self.next_entity..];

        // Find the next matching entity
        let entity_id = mapping
            .iter()
            .enumerate()
            .find_map(|(index, entity)| {
                if *entity {
                    let curr = self.next_entity + index;

                    // Verify that entity has requested components 
                    if S::is_viable(curr, self.components) && F::filter(curr, self.components) {
                        return Some(curr)
                    }

                    None
                } else {
                    // Entity ID is not in use
                    None
                }
            })?;

        self.next_entity = entity_id + 1;
        if S::IS_ENTITY {
            // S is Entity
            if S::READONLY {
                let entity = Entity {
                    entities: self.entities,
                    components: self.components,
                    id: EntityId(entity_id)
                };
    
                let transmuted = unsafe {
                    mem::transmute_copy(&entity)
                };
                mem::forget(entity);
    
                return Some(transmuted)
            } 
            
            // S is EntityMut
            else {
                todo!();
            }
        } else {
            if S::READONLY {
                S::fetch(entity_id, self.components)
            } else {
                todo!();
            }
        }
    }
}