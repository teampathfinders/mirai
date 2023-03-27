use std::{any::{Any, TypeId}, collections::HashMap};

use dashmap::DashMap;

pub trait Spawnable {
    fn insert_all(self, storage: &mut Components, entity: usize);
}

pub trait Component: Send + Sync {

}

impl<T: Component + 'static> Spawnable for T {
    fn insert_all(self, storage: &mut Components, entity: usize) {
        storage.insert(self, entity);
    }
}
impl<T0: Component + 'static, T1: Component + 'static> Spawnable for (T0, T1) {
    fn insert_all(self, storage: &mut Components, entity: usize) {
        storage.insert(self.0, entity);
        storage.insert(self.1, entity);
    }
}

pub trait Store {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct SpecializedStore<T: Component> {
    mapping: HashMap<usize, usize>,
    storage: Vec<Option<T>>
}

impl<T: Component> SpecializedStore<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, entity: usize) -> Option<&T> {
        let mapped = self.mapping.get(&entity)?;
        self.storage.get(*mapped)?.as_ref()
    }

    pub fn insert(&mut self, component: T, entity: usize) {
        let index = self.storage
            .iter_mut()
            .enumerate()
            .find_map(|(i, v)| if v.is_none() {
                Some(i)
            } else {
                None
            });

        if let Some(index) = index {
            self.mapping.insert(entity, index);
            self.storage[index] = Some(component);
        } else {
            let len = self.storage.len();
            self.mapping.insert(entity, len);
            self.storage.push(Some(component));
        }
    }
}

impl<T: Component> Default for SpecializedStore<T> {
    fn default() -> Self {
        Self {
            mapping: HashMap::new(),
            storage: Vec::new()
        }
    }
}

impl<T: Component + 'static> Store for SpecializedStore<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Default)]
pub struct Components {
    storage: HashMap<TypeId, Box<dyn Store + Send + Sync>>
}

impl Components {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<T: Component + 'static>(&mut self, component: T, entity: usize) {
        let mut entry = self.storage
            .entry(TypeId::of::<T>())
            .or_insert_with(|| {
                Box::new(SpecializedStore::<T>::new())
            });

        let downcast = entry.as_any_mut().downcast_mut::<SpecializedStore<T>>().unwrap();
        downcast.insert(component, entity);
    }

    pub fn entity_has<T: Component + 'static>(&self, entity: usize) -> bool {
        if let Some(upcasted) = self.storage.get(&TypeId::of::<T>()) {
            if let Some(store) = upcasted.as_any().downcast_ref::<SpecializedStore<T>>() {
                return store.mapping.contains_key(&entity)
            }
        }

        false
    }

    pub fn get<T: Component + 'static>(&self, entity: usize) -> Option<&T> {
        let storage = self.storage.get(&TypeId::of::<T>())?;
        let downcast = storage.as_any().downcast_ref::<SpecializedStore<T>>()?;

        downcast.get(entity)
    }
}