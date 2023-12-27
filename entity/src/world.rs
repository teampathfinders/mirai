use crate::component::Components;
use crate::entity::Entities;
use crate::system::Systems;

pub struct World {
    entities: Entities,
    components: Components,
    systems: Systems,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Entities::new(),
            components: Components::new(),
            systems: Systems::new(),
        }
    }
}
