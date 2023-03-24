use std::{collections::HashMap, any::{TypeId, Any}, hash::Hash};

use crate::{system::{System, IntoSystem}, component::{Insertable, Component}};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct EntityId(usize);

pub struct Entity<'world> {
    world: &'world mut World,
    id: EntityId
}

trait Store {
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct SpecificStore<T>
where
    T: Component
{
    mapping: HashMap<EntityId, usize>,
    storage: Vec<T>
}

impl<T> SpecificStore<T>
where
    T: Component,
{
    pub fn insert(&mut self, component: T) {
        todo!();
    }
}

impl<T> Store for SpecificStore<T> 
where
    T: Component + 'static
{
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl<T> Default for SpecificStore<T> 
where
    T: Component
{
    fn default() -> SpecificStore<T> {
        SpecificStore {
            mapping: HashMap::new(),
            storage: Vec::new()
        }
    }
}

pub struct ComponentStore {
    storage: HashMap<TypeId, Box<dyn Store>>
}

impl ComponentStore {
    pub fn new() -> ComponentStore {
        ComponentStore::default()
    }

    pub fn insert<T>(&mut self, data: T, owner: EntityId) 
    where
        T: Component + 'static
    {
        let ty = TypeId::of::<T>();
        let entry = self.storage.entry(ty)
            .or_insert_with(|| Box::new(SpecificStore::<T>::default()));

        let downcast: &mut SpecificStore<T> = entry.as_any_mut().downcast_mut().unwrap();
        downcast.insert(data);
    }
}

impl Default for ComponentStore {
    fn default() -> ComponentStore {
        ComponentStore {
            storage: HashMap::new()
        }
    }
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
    components: ComponentStore,
    executor: Executor
}

impl World {
    pub fn new() -> World {
        World::default()
    }

    pub fn spawn(&mut self, components: impl Insertable) -> Entity {
        // components.store_all(&mut self.components);
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
            components: ComponentStore::default(),
            executor: Executor::default()
        }
    }
}