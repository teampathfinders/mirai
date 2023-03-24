use std::{collections::HashMap, any::{TypeId, Any}};

use crate::EntityId;

pub trait Component {

}

impl<T> Component for &T where T: Component {}
impl<T> Component for &mut T where T: Component {}

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

pub trait Requestable {
    
}

impl<T> Requestable for T
where
    T: Component,
{

}

impl<T0, T1> Requestable for (T0, T1)
where
    T0: Component,
    T1: Component,
{

}

trait Store {
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct SpecificStore<T>
where
    T: Component
{
    mapping: HashMap<usize, usize>,
    storage: Vec<Option<T>>
}

impl<T> SpecificStore<T>
where
    T: Component,
{
    pub fn insert(&mut self, owner: usize, component: T) {
        let gap = self.storage
            .iter()
            .enumerate()
            .find_map(|(i, o)| if o.is_none() { Some(i) } else { None });

        if let Some(idx) = gap {
            // Fill in gaps to keep array as packed as possible.
            self.mapping.insert(owner, idx);
            self.storage[idx] = Some(component);
        } else {
            let idx = self.storage.len();
            self.mapping.insert(owner, idx);
            self.storage.push(Some(component));
        }
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

    pub fn insert<T>(&mut self, data: T, owner: usize) 
    where
        T: Component + 'static
    {
        let ty = TypeId::of::<T>();
        let entry = self.storage.entry(ty)
            .or_insert_with(|| Box::new(SpecificStore::<T>::default()));

        let downcast: &mut SpecificStore<T> = entry.as_any_mut().downcast_mut().unwrap();
        downcast.insert(owner, data);
    }
}

impl Default for ComponentStore {
    fn default() -> ComponentStore {
        ComponentStore {
            storage: HashMap::new()
        }
    }
}