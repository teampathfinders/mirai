use std::{sync::Arc, marker::PhantomData};

use crate::{private, component::Component, entity::{Entity, EntityId}, world::WorldState};

pub trait RefComponent<'a>: Sized {
    type NonRef: Component + 'static;

    const MUTABLE: bool;    

    fn fetch(entity: usize, state: &'a WorldState) -> Self {
        unimplemented!()
    }
}

impl<'a, T: RefComponent<'a>> private::Sealed for T {}
impl<'a, T0: RefComponent<'a>, T1: RefComponent<'a>> private::Sealed for (T0, T1) {}

impl<'a, T> RefComponent<'a> for &'a T 
where
    T: Component + 'static
{
    type NonRef = T;

    const MUTABLE: bool = false;

    fn fetch(entity: usize, state: &WorldState) -> Self {
        state.components.get(entity)
    }
}

impl<'a, T> RefComponent<'a> for &'a mut T 
where
    T: Component + 'static,
{
    type NonRef = T;

    const MUTABLE: bool = true;
}

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
pub unsafe trait Requestable<'a>: Sized + private::Sealed {
    /// This is here because ReqIter logic requires checking whether the requestable is an entity.
    /// It cannot be done using TypeId as that requires a static lifetime, which we do not have.
    const IS_ENTITY: bool;
    const MUTABLE: bool;

    fn fetch(entity: usize, state: &WorldState) -> Self {
        unimplemented!()
    }

    fn matches(entity: usize, state: &WorldState) -> bool;
}

unsafe impl<'a, T> Requestable for T 
where
    T: RefComponent<'a>,
{  
    const IS_ENTITY: bool = false;
    const MUTABLE: bool = T::MUTABLE;

    fn fetch(entity: usize, state: &WorldState) -> Self {
        <T as RefComponent>::fetch(entity, state)
    }

    fn matches(entity: usize, state: &WorldState) -> bool {
        state.entity_has::<T::NonRef>(entity)
    }
}

// unsafe impl<T0, T1> Requestable for (T0, T1) 
// where
//     T0: RefComponent,
//     T1: RefComponent,
// {
//     type Item =

//     const IS_ENTITY: bool = false;
//     const MUTABLE: bool = T0::MUTABLE || T1::MUTABLE;

//     fn matches(entity: usize, state: &WorldState) -> bool {
//         state.entity_has::<T0::NonRef>(entity) && state.entity_has::<T1::NonRef>(entity)
//     }
// }

unsafe impl Requestable for Entity {
    const IS_ENTITY: bool = true;
    const MUTABLE: bool = false;

    fn matches(entity: usize, state: &WorldState) -> bool {
        true
    }
}

pub struct Request<S, F = ()>
where
    S: Requestable,
    F: Filters,
{   
    world_state: Arc<WorldState>,
    _marker: PhantomData<(S, F)>
}

impl<S, F> Request<S, F> 
where
    S: Requestable,
    F: Filters
{
    pub fn new(world_state: Arc<WorldState>) -> Self {
        Self {
            world_state, _marker: PhantomData
        }
    }
}

impl<'q, S, F> IntoIterator for &'q Request<S, F> 
where
    S: Requestable,
    F: Filters
{
    type IntoIter = ReqIter<'q, S, F>;
    type Item = S;

    fn into_iter(self) -> Self::IntoIter {
        ReqIter { 
            next_index: 0,
            state: &self.world_state,
            _marker: PhantomData
        }
    }
}

pub struct ReqIter<'q, S: Requestable, F: Filters> {
    next_index: usize,
    state: &'q Arc<WorldState>,
    _marker: PhantomData<(S, F)>
}

impl<'q, S: Requestable, F: Filters> Iterator for ReqIter<'q, S, F> {
    type Item = S;

    fn next(&mut self) -> Option<S> {
        let entity_id = self.state.entities.mapping.read()[self.next_index..]
            .iter()
            .enumerate()
            .find_map(|(i, v)| if *v {
                let entity_id = self.next_index + i;
                if S::matches(entity_id, self.state) && F::filter(entity_id, self.state) {
                    Some(entity_id)
                } else {
                    None
                }
            } else {
                None
            })?;

        self.next_index = entity_id + 1;
        if S::IS_ENTITY {
            let entity = Entity {
                world_state: Arc::clone(self.state),
                id: EntityId(entity_id)
            };

            // This would probably benefit from specialisation of the `S` generic, which is currently
            // unstable.
            // SAFETY: This is safe because `S` is guaranteed to be of type `Entity`.
            // The caller has to make sure that `IS_ENTITY` is only true for `Entity`.
            let transmuted = unsafe {
                std::mem::transmute_copy(&entity)
            };
            std::mem::forget(entity);

            Some(transmuted)
        } else {
            if S::MUTABLE {
                todo!();
            } else {
                let out = S::fetch(entity_id, self.state);
                Some(out)
            }
        }
    }
}