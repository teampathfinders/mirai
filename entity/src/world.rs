use crate::component::{Component, Components};
use crate::entity::{Entities, Entity, EntityId, EntityMut};
use crate::system::Systems;

pub trait ComponentBundle {
    fn insert_into(self, entity: EntityId, store: &mut Components);
}

impl<C: Component> ComponentBundle for C {
    #[inline]
    fn insert_into(self, entity: EntityId, store: &mut Components) {
        store.insert(entity, self);
    }
}
impl<C1: Component, C2: Component> ComponentBundle for (C1, C2) {
    #[inline]
    fn insert_into(self, entity: EntityId, store: &mut Components) {
        store.insert(entity, self.0);
        store.insert(entity, self.1);
    }
}
impl<C1: Component, C2: Component, C3: Component> ComponentBundle for (C1, C2, C3) {
    #[inline]
    fn insert_into(self, entity: EntityId, store: &mut Components) {
        store.insert(entity, self.0);
        store.insert(entity, self.1);
        store.insert(entity, self.2);
    }
}

#[derive(Debug)]
pub struct World {
    pub(crate) entities: Entities,
    pub(crate) components: Components,
    pub(crate) systems: Systems,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Entities::new(),
            components: Components::new(),
            systems: Systems::new(),
        }
    }

    pub fn spawn(&mut self, bundle: impl ComponentBundle) -> EntityMut {
        let id = self.entities.request_id();
        bundle.insert_into(id, &mut self.components);

        EntityMut { id, world: self }
    }

    pub fn get(&self, id: EntityId) -> Option<Entity> {
        if self.entities.is_spawned(id) {
            Some(Entity { id, world: self })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<EntityMut> {
        if self.entities.is_spawned(id) {
            Some(EntityMut { id, world: self })
        } else {
            None
        }
    }
}
