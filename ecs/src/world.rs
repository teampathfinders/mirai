use std::{collections::HashMap, any::{TypeId, Any}, hash::Hash};

use crate::{system::{System, IntoSystem}, component::{Spawnable, Component, ComponentStore}, Entity, entity::EntityStore, EntityId};

pub struct Executor {
    systems: Vec<Box<dyn System>>
}

impl Executor {
    pub fn new() -> Executor {
        Executor::default()
    }

    pub fn schedule(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }
}

impl Default for Executor {
    fn default() -> Executor {
        Executor {
            systems: Vec::new()
        }
    }
}

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

    pub fn system<Params>(&mut self, system: impl IntoSystem<Params>) {
        let system = system.into_system();
        self.executor.schedule(system);
    }
}

impl Default for World {
    fn default() -> World {
        World {
            entities: EntityStore::default(),
            components: ComponentStore::default(),
            executor: Executor::default()
        }
    }
}