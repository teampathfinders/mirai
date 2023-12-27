use std::ops::Deref;
use crate::component::Component;
use crate::world::World;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct EntityId(pub(crate) usize);

#[derive(Debug, Copy, Clone)]
pub struct Entity<'w> {
    pub(crate) id: EntityId,
    pub(crate) world: &'w World,
}

impl<'w> Entity<'w> {
    #[inline]
    pub fn id(&self) -> EntityId {
        self.id
    }

    #[inline]
    pub fn get<T: Component + 'static>(&self) -> Option<&T> {
        self.world.components.get(self.id)
    }
}

#[derive(Debug)]
pub struct EntityMut<'w> {
    pub(crate) id: EntityId,
    pub(crate) world: &'w mut World,
}

impl<'w> EntityMut<'w> {
    #[inline]
    pub fn id(&self) -> EntityId {
        self.id
    }

    #[inline]
    pub fn freeze(self) -> Entity<'w> {
        Entity { id: self.id, world: self.world }
    }

    #[inline]
    pub fn despawn(self) {
        self.world.components.despawn(self.id);
    }

    #[inline]
    pub fn get<T: Component + 'static>(&self) -> Option<&T> {
        self.world.components.get(self.id)
    }

    #[inline]
    pub fn get_mut<T: Component + 'static>(&mut self) -> Option<&mut T> {
        self.world.components.get_mut(self.id)
    }

    #[inline]
    pub fn component<T: Component + 'static>(&mut self, component: T) -> Option<T> {
        self.world.components.insert(self.id, component)
    }
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
