use std::{ops::{Deref, DerefMut}, marker::PhantomData, sync::Arc, mem};

use parking_lot::RwLock;

use crate::{component::{Components, Component}, private, entity::{Entity, EntityId}, world::WorldState};

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
pub unsafe trait Requestable<'f>: Sized + private::Sealed {
    type Fetch: Component + 'static;
    type Item: Requestable<'f>;

    /// This is here because ReqIter logic requires checking whether the requestable is an entity.
    /// It cannot be done using TypeId as that requires a static lifetime, which we do not have.
    const IS_ENTITY: bool;
    const READONLY: bool;

    fn is_viable(entity: usize, components: &Components) -> bool;

    fn fetch(entity: usize, components: &'f Components) -> Option<Self::Item> {
        unimplemented!()
    }
}

impl<T> private::Sealed for &T where T: Component {}

unsafe impl<'f, T> Requestable<'f> for &T 
where
    T: Component + 'static
{
    type Fetch = T;
    type Item = &'f T;

    const IS_ENTITY: bool = false;
    const READONLY: bool = true;

    #[inline]
    fn is_viable(entity: usize, components: &Components) -> bool {
        components.entity_has::<T>(entity)
    }

    fn fetch(entity: usize, components: &'f Components) -> Option<Self::Item> {
        components.get(entity)
    }
}

impl private::Sealed for Entity {}

unsafe impl<'f> Requestable<'f> for Entity {
    type Fetch = ();
    type Item = Entity;

    const IS_ENTITY: bool = true;
    const READONLY: bool = true;

    #[inline]
    fn is_viable(entity: usize, components: &Components) -> bool {
        true
    }
}

pub struct Request<'f, S, F = ()> 
where
    S: Requestable<'f>,
    F: Filters
{
    next_entity: usize,
    state: Arc<RwLock<WorldState>>,
    _marker: PhantomData<&'f (S, F)>
}

pub trait RequestCheck {}

impl RequestCheck for (Entity, &()) {}
impl<A> RequestCheck for (A, A) {}

impl<'f, S, F> Request<'f, S, F> 
where
    S: Requestable<'f>,
    F: Filters,
{
    pub fn new(state: Arc<RwLock<WorldState>>) -> Self {
        Self {
            next_entity: 0,
            state, _marker: PhantomData
        }
    }

    pub fn next(&mut self) -> Option<S::Item> {
        let lock = self.state.read();
        let entity = {
            let entities = &lock.entities.mapping.read()[self.next_entity..];
            let entity = entities
                .iter()
                .enumerate()
                .find_map(|(index, entity)| {
                    if *entity {
                        let curr = self.next_entity + index;
                        if S::is_viable(curr, &lock.components) && F::filter(curr, &lock.components) {
                            return Some(curr)
                        }
                    }   
    
                    None
                })?;
    
            self.next_entity = entity + 1;
            entity
        };

        if S::IS_ENTITY {
            let out = Entity {
                state: self.state.clone(),
                id: EntityId(entity)
            };

            // SAFETY: The `S` generic and `Entity` type are guaranteed to be equal due to the IS_ENTITY check.
            let cast = unsafe {
                mem::transmute_copy(&out)
            };
            mem::forget(out);

            Some(cast)
        } else if S::READONLY {
            lock.components.get::<S::Fetch>(entity)

            // // SAFETY: The `S` generic and the type of `out` are guaranteed to be equal.
            // let cast = unsafe {
            //     mem::transmute_copy(&out)
            // };

            // s
        } else {
            todo!();
        }
    }
}



