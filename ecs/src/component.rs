use std::{collections::HashMap, any::{TypeId, Any}};

use crate::EntityId;

pub trait Component {

}

pub trait RefComponent {
    const SHAREABLE: bool;
}

impl<T> RefComponent for &T 
where
    T: Component
{
    const SHAREABLE: bool = true;
}

impl<T> RefComponent for &mut T 
where
    T: Component
{
    const SHAREABLE: bool = false;
}

pub trait Spawnable {
    fn store_all(self, owner: usize, store: &mut ComponentStore);
}

impl<T> Spawnable for T 
where 
    T: Component + 'static 
{
    fn store_all(self, owner: usize, store: &mut ComponentStore) {
        store.insert(self, owner);
    }
}

impl<T0, T1> Spawnable for (T0, T1) 
where 
    T0: Component + 'static, 
    T1: Component + 'static 
{
    fn store_all(self, owner: usize, store: &mut ComponentStore) {
        store.insert(self.0, owner);
        store.insert(self.1, owner);
    }
}

trait Store {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn release_entity(&mut self, entity: usize);
}

pub struct SpecializedStore<T>
where
    T: Component
{
    mapping: HashMap<usize, usize>,
    storage: Vec<Option<T>>
}

impl<T> SpecializedStore<T>
where
    T: Component,
{
    pub fn insert(&mut self, owner: usize, component: T) {
        for (i, s) in self.storage.iter_mut().enumerate() {
            if s.is_none() {
                *s = Some(component);
                self.mapping.insert(owner, i);
                return
            }
        }

        let idx = self.storage.len();
        self.mapping.insert(owner, idx);
        self.storage.push(Some(component));
    }
}

impl<T> Store for SpecializedStore<T> 
where
    T: Component + 'static
{
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn release_entity(&mut self, entity: usize) {
        if let Some(idx) = self.mapping.remove(&entity) {
            self.storage[idx] = None;
        }
    }
}

impl<T> Default for SpecializedStore<T> 
where
    T: Component
{
    fn default() -> SpecializedStore<T> {
        SpecializedStore {
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

    pub fn insert<T>(&mut self, data: T, owner: usize) 
    where
        T: Component + 'static
    {
        let ty = TypeId::of::<T>();
        let entry = self.storage.entry(ty)
            .or_insert_with(|| Box::new(SpecializedStore::<T>::default()));

        let downcast: &mut SpecializedStore<T> = entry.as_any_mut().downcast_mut().unwrap();
        downcast.insert(owner, data);
    }

    pub fn release_entity(&mut self, entity: usize) {
        self.storage.iter_mut().for_each(|(_, v)| v.release_entity(entity));
    }
}

impl Default for ComponentStore {
    fn default() -> ComponentStore {
        ComponentStore {
            storage: HashMap::new()
        }
    }
}