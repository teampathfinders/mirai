use crate::world::World;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct EntityId(pub(crate) usize);

#[derive(Debug, Copy, Clone)]
pub struct Entity<'w> {
    id: EntityId,
    world: &'w World,
}

#[derive(Debug)]
pub struct EntityMut<'w> {
    pub(crate) id: EntityId,
    pub(crate) world: &'w mut World,
}

#[derive(Debug)]
pub struct Entities {
    next_index: usize,
}

impl Entities {
    pub fn new() -> Self {
        Self { next_index: 0 }
    }

    pub fn request_id(&mut self) -> EntityId {
        self.next_index += 1;
        EntityId(self.next_index - 1)
    }
}
