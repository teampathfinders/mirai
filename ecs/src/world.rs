use std::{collections::HashMap, any::{TypeId, Any}, hash::Hash};

use crate::{system::{System, IntoSystem, Executor, SharedSystem}, component::{Spawnable, Component, ComponentStore}, Entity, entity::EntityStore, EntityId};

#[derive(Default)]
pub struct World<'w> {
    entities: EntityStore,
    components: ComponentStore<'w>,
    executor: Executor<'w>
}

impl<'w> World<'w> {
    pub fn new() -> World<'w> {
        World::default()
    }

    pub fn spawn(&'w mut self, components: impl Spawnable<'w>) -> EntityId
    {
        let entity_id = self.entities.acquire();
        components.store_all(entity_id, &mut self.components);

        EntityId(entity_id)
    }

    pub fn despawn(&'w mut self, entity: EntityId) {
        self.components.release_entity(entity.0);
        self.entities.release(entity.0);
    }

    pub fn system<Sys, Params>(&'w mut self, system: impl IntoSystem<'w, Sys, Params>)
    where
        Sys: SharedSystem<'w, Params>,
        Params: 'w
    {
        let system = system.into_system();
        todo!();
        // self.executor.schedule(system);
    }

    pub(crate) fn execute(&'w mut self) {
        self.executor.execute(self);
    }
}