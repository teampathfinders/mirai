use crate::World;

#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct EntityId(pub(crate) usize);

impl From<Entity<'_>> for EntityId {
    fn from(entity: Entity) -> Self {
        entity.id()
    }
}

pub struct Entity<'w> {
    pub(crate) world: &'w mut World,
    pub(crate) id: EntityId
}

impl<'w> Entity<'w> {
    #[inline]
    pub fn id(self) -> EntityId {
        self.id
    }

    pub fn despawn(self) {
        self.world.despawn(self.id);
    }
}

pub struct EntityStore {
    mapping: Vec<bool>
}

impl EntityStore {
    pub fn new() -> EntityStore {
        EntityStore::default()
    }

    pub fn acquire(&mut self) -> usize {
        self.mapping
            .iter_mut()
            .enumerate()
            .find_map(|(i, v)| if *v {
                None
            } else {
                *v = true;
                Some(i)
            })
            .or_else(|| {
                let idx = self.mapping.len();
                self.mapping.push(true);
                Some(idx)
            })
            .unwrap()
    }    

    pub fn release(&mut self, idx: usize) {
        self.mapping[idx] = false;
    }
}

impl Default for EntityStore {
    fn default() -> EntityStore {
        EntityStore {
            mapping: Vec::new()
        }
    }
}