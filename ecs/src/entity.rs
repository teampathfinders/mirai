use std::sync::Arc;

use parking_lot::RwLock;

use crate::{private, world::WorldState, component::Component};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct EntityId(pub(crate) usize);

pub struct Entity {
    pub(crate) world_state: Arc<WorldState>,
    pub(crate) id: EntityId,
}

impl private::Sealed for Entity {}

impl Entity {
    pub fn id(&self) -> &EntityId {
        &self.id
    }

    pub fn despawn(self) {
        todo!();
    }

    #[inline]
    pub fn has_component<T: Component + 'static>(&self) -> bool {
        self.world_state.components.entity_has::<T>(self.id.0)
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