use std::{sync::Arc, marker::PhantomData};

use crate::{private, component::Component, entity::{Entity, EntityId}, world::WorldState};

pub trait RefComponent {
    const MUTABLE: bool;    
}

impl<T: RefComponent> private::Sealed for T {}
impl<T0: RefComponent, T1: RefComponent> private::Sealed for (T0, T1) {}

impl<T: Component> RefComponent for &T {
    const MUTABLE: bool = false;
}

impl<T: Component> RefComponent for &mut T {
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

pub unsafe trait Requestable: private::Sealed {
    /// This is here because ReqIter logic requires checking whether the requestable is an entity.
    /// It cannot be done using TypeId as that requires a static lifetime, which we do not have.
    const IS_ENTITY: bool;
    const MUTABLE: bool;
}

unsafe impl<T: RefComponent> Requestable for T {
    const IS_ENTITY: bool = false;
    const MUTABLE: bool = T::MUTABLE;
}

unsafe impl<T0: RefComponent, T1: RefComponent> Requestable for (T0, T1) {
    const IS_ENTITY: bool = false;
    const MUTABLE: bool = T0::MUTABLE || T1::MUTABLE;
}

unsafe impl Requestable for Entity {
    const IS_ENTITY: bool = true;
    const MUTABLE: bool = false;
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
                if F::filter(entity_id, self.state) {
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
            None
        }
    }
}