use crate::{entity::EntityStore, component::ComponentStore, system::{Executor, IntoSystem, SystemParam, SystemParams}, Spawnable, EntityId, Entity};


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

    pub fn spawn(&mut self, spawnable: impl Spawnable) -> Entity {
        let entity = self.entities.acquire();

        Entity {
            id: EntityId(entity),
            world: self   
        }
    }

    pub fn despawn(&mut self, entity: impl Into<EntityId>) {
        let entity = entity.into();
        let id = entity.0;

        self.components.release_entity(id);
        self.entities.release(id);
    }

    pub fn system<S, P>(&mut self, system: impl IntoSystem<S, P>) 
    where
        P: SystemParams
    {
        self.executor.add_system(system);
    }
}