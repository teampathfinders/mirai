use crate::{system::{System, IntoSystem}, component::ComponentCollection};

pub struct EntityId(usize);

pub struct Entity<'world> {
    world: &'world mut World,
    id: EntityId
}

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
    executor: Executor
}

impl World {
    pub fn new() -> World {
        World::default()
    }

    pub fn spawn(&mut self, components: impl ComponentCollection) -> Entity {
        todo!();
    }

    pub fn schedule<Params>(&mut self, system: impl IntoSystem<Params>) {
        let system = system.into_system();
        self.executor.schedule(system);
    }
}

impl Default for World {
    fn default() -> World {
        World {
            executor: Executor::default()
        }
    }
}