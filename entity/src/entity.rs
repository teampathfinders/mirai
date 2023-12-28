use crate::component::Component;
use crate::world::World;
use bitvec::macros::internal::funty::Fundamental;
use bitvec::vec::BitVec;
use std::ops::Deref;

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
        self.world.entities.despawn(self.id);
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
    active: BitVec,
}

impl Entities {
    pub fn new() -> Self {
        Self { active: BitVec::new() }
    }

    pub fn despawn(&mut self, id: EntityId) {
        self.active.set(id.0, false);
    }

    /// Checks whether an entity is actually alive.
    pub fn is_spawned(&self, id: EntityId) -> bool {
        self.active.get(id.0).map(|b| *b).unwrap_or(false)
    }

    pub fn request_id(&mut self) -> EntityId {
        let reused_id = self.active.iter().by_vals().enumerate().find_map(|(i, v)| if v { None } else { Some(i) });

        let id = if let Some(reused) = reused_id {
            self.active.set(reused, true);
            reused
        } else {
            self.active.push(true);
            self.active.len() - 1
        };

        EntityId(id)
    }
}
