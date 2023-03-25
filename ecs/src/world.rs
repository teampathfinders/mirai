use std::{collections::HashMap, any::{TypeId, Any}, hash::Hash};

use crate::{system::{System, IntoSystem, Executor}, component::{Spawnable, Component, ComponentStore}, Entity, entity::EntityStore, EntityId};

#[derive(Default)]
pub struct World {
    entities: EntityStore,
    components: ComponentStore,
    executor: Executor
}

impl World {
    pub fn new() -> World {
        World::default()
    }

    pub fn spawn(&mut self, components: impl Spawnable) -> Entity {
        let entity_id = self.entities.acquire();
        components.store_all(entity_id, &mut self.components);

        // components.store_all(&mut self.components);
        Entity {
            id: EntityId(entity_id),
            world: self
        }
    }

    pub fn despawn(&mut self, entity: EntityId) {
        self.components.release_entity(entity.0);
        self.entities.release(entity.0);
    }

    pub fn system<Params>(&mut self, system: impl IntoSystem<Params>) {
        let system = system.into_system();
        self.executor.schedule(system);
    }

    pub(crate) fn execute(&mut self) {
        self.executor.execute(self);
    }
}