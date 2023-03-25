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

    pub fn spawn<'a>(&mut self, spawnable: impl Spawnable<'a, 'w>) -> EntityId {
        let id = self.entities.acquire();

        EntityId(id)
    }

    pub fn despawn(&'w mut self, entity: EntityId) {
        self.components.release_entity(entity.0);
        self.entities.release(entity.0);
    }

    pub fn system<Sys, Params>(&mut self, system: impl IntoSystem<'w, Sys, Params>)
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