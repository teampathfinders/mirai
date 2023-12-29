use crate::component::{Component, Components};
use crate::entity::{Entities, Entity, EntityId, EntityMut};
use crate::system::{SysContainer, NakedSys, Sys, Systems, SysParamBundle};

pub trait ComponentBundle {
    fn insert_into(self, entity: EntityId, store: &mut Components);
}

impl ComponentBundle for () {
    fn insert_into(self, entity: EntityId, store: &mut Components) {}
}

impl<C: Component> ComponentBundle for C {
    fn insert_into(self, entity: EntityId, store: &mut Components) {
        store.insert(entity, self);
    }
}

impl<C1: Component, C2: Component> ComponentBundle for (C1, C2) {
    fn insert_into(self, entity: EntityId, store: &mut Components) {
        store.insert(entity, self.0);
        store.insert(entity, self.1);
    }
}

impl<C1: Component, C2: Component, C3: Component> ComponentBundle for (C1, C2, C3) {
    fn insert_into(self, entity: EntityId, store: &mut Components) {
        store.insert(entity, self.0);
        store.insert(entity, self.1);
        store.insert(entity, self.2);
    }
}

pub struct World<'w> {
    pub(crate) entities: Entities,
    pub(crate) components: Components,
    pub(crate) systems: Systems<'w>,
}

impl<'w> World<'w> {
    pub fn new() -> Self {
        Self {
            entities: Entities::new(),
            components: Components::new(),
            systems: Systems::new(),
        }
    }

    pub fn spawn<'a>(&'a mut self, bundle: impl ComponentBundle) -> EntityMut<'a, 'w> {
        let id = self.entities.request_id();
        bundle.insert_into(id, &mut self.components);

        EntityMut { id, world: self }
    }

    #[inline]
    pub fn spawn_empty(&'w mut self) -> EntityMut {
        self.spawn(())
    }

    pub fn system<P, S>(&mut self, system: S)
    where
        P: SysParamBundle + 'w,
        S: NakedSys<'w, P>,
        SysContainer<'w, P, S>: Sys<'w>
    {
        self.systems.insert(system);
    }

    pub fn tick(&mut self) {
        self.systems.tick(&self.components);
    }

    pub fn get<'a>(&'a self, id: EntityId) -> Option<Entity<'a, 'w>> {
        Some(Entity { id, world: self })
    }

    pub fn get_mut<'a>(&'a mut self, id: EntityId) -> Option<EntityMut<'a, 'w>> {
        Some(EntityMut { id, world: self })
    }
}
