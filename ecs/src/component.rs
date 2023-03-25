use std::{collections::HashMap, any::{TypeId, Any}, marker::PhantomData};

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

pub trait Spawnable<'w> {
    fn store_all(self, owner: usize, store: &'w mut ComponentStore<'w>);
}

impl<'w, T> Spawnable<'w> for T 
where 
    T: Component + 'w 
{
    fn store_all(self, owner: usize, store: &'w mut ComponentStore<'w>) {
        store.insert(self, owner);
    }
}

impl<'w, T0, T1> Spawnable<'w> for (T0, T1) 
where 
    T0: Component + 'w,
    T1: Component + 'w
{
    fn store_all(self, owner: usize, store: &'w mut ComponentStore<'w>) {
        store.insert(self.0, owner);
        store.insert(self.1, owner);
    }
}

trait Store<'w> {
    // fn as_any_mut(&mut self) -> &mut dyn Any;
    fn release_entity(&mut self, entity: usize);
}

pub struct SpecializedStore<'w, T>
where
    T: Component + 'w
{
    mapping: HashMap<usize, usize>,
    storage: Vec<Option<T>>,
    _marker: PhantomData<&'w ()>
}

impl<'w, T> SpecializedStore<'w, T>
where
    T: Component + 'w,
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

impl<'w, T> Store<'w> for SpecializedStore<'w, T> 
where
    T: Component + 'w
{
    fn release_entity(&mut self, entity: usize) {
        if let Some(idx) = self.mapping.remove(&entity) {
            self.storage[idx] = None;
        }
    }
}

impl<'w, T> Default for SpecializedStore<'w, T> 
where
    T: Component
{
    fn default() -> SpecializedStore<'w, T> {
        SpecializedStore {
            mapping: HashMap::new(),
            storage: Vec::new(),
            _marker: PhantomData
        }
    }
}

pub struct ComponentStore<'w> {
    storage: HashMap<TypeId, Box<dyn Store<'w>>>
}

impl<'w> ComponentStore<'w> {
    pub fn new() -> ComponentStore<'w> {
        ComponentStore::default()
    }

    pub fn insert<T>(&mut self, data: T, owner: usize) 
    where
        T: Component + 'w
    {
        // let ty = TypeId::of::<T>();
        // let entry = self.storage.entry(ty)
        //     .or_insert_with(|| Box::new(SpecializedStore::<T>::default()));

        todo!();
        // let downcast: &mut SpecializedStore<T> = entry.as_any_mut().downcast_mut().unwrap();
        // downcast.insert(owner, data);
    }

    pub fn release_entity(&mut self, entity: usize) {
        self.storage.iter_mut().for_each(|(_, v)| v.release_entity(entity));
    }
}

impl<'w> Default for ComponentStore<'w> {
    fn default() -> ComponentStore<'w> {
        ComponentStore {
            storage: HashMap::new()
        }
    }
}