use std::{sync::Arc, ops::Deref};

use parking_lot::RwLock;

use crate::{private, world::WorldState, component::{Component, Components}};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct EntityId(pub(crate) usize);

pub struct EntityMut<'state> {
    pub(crate) entities: &'state mut Entities,
    pub(crate) components: &'state mut Components,
    pub(crate) id: EntityId
}

impl EntityMut<'_> {
    pub fn despawn(self) {
        todo!();
    }
}

pub struct Entity<'state> {
    pub(crate) entities: &'state Entities,
    pub(crate) components: &'state Components,
    pub(crate) id: EntityId,
}

impl Entity<'_> {
    pub fn id(&self) -> &EntityId {
        &self.id
    }

    #[inline]
    pub fn has_component<T: Component + 'static>(&self) -> bool {
        self.components.entity_has::<T>(self.id.0)
    }
}

#[derive(Default)]
pub struct Entities {
    pub(crate) mapping: RwLock<Vec<bool>>
}

impl Entities {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn acquire(&self) -> usize {
        let mut lock = self.mapping.write();
        lock
            .iter_mut()
            .enumerate()
            .find_map(|(i, v)| {
                if *v {
                    None
                } else {
                    *v = true;
                    Some(i)
                }
            })
            .or_else(|| {
                let len = lock.len();
                lock.push(true);
                Some(len)
            })
            .unwrap()
    }

    pub fn release(&self, entity: usize) {
        self.mapping.write()[entity] = false;
    }
}