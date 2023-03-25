use std::{collections::HashMap, any::{TypeId, Any}, marker::PhantomData};
use better_any::TidAble;

use better_any::Tid;

use crate::EntityId;

pub trait Component<'t>: Tid<'t> {

}

pub trait RefComponent {
    const SHAREABLE: bool;
}

impl<'t, T> RefComponent for &T 
where
    T: Component<'t>
{
    const SHAREABLE: bool = true;
}

impl<'t, T> RefComponent for &mut T 
where
    T: Component<'t>
{
    const SHAREABLE: bool = false;
}

pub trait Spawnable {
    fn store_all(self, owner: usize, store: &mut ComponentStore);
}

impl<'t, T> Spawnable for T 
where 
    T: Component<'t>
{
    fn store_all(self, owner: usize, store: &mut ComponentStore) {
        store.insert(self, owner);
    }
}

impl<'t, T0, T1> Spawnable for (T0, T1) 
where 
    T0: Component<'t>,
    T1: Component<'t>
{
    fn store_all(self, owner: usize, store: &mut ComponentStore) {
        store.insert(self.0, owner);
        store.insert(self.1, owner);
    }
}

trait Store {
    // fn as_any_mut(&mut self) -> &mut dyn Any;
    fn release_entity(&mut self, entity: usize);
}

pub struct SpecializedStore<'t, T>
where
    T: Component<'t>,
{
    mapping: HashMap<usize, usize>,
    storage: Vec<Option<T>>,
    _marker: PhantomData<&'t T>
}

impl<'t, T> SpecializedStore<'t, T>
where
    T: Component<'t>,
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

impl<'t, T> Store for SpecializedStore<'t, T> 
where
    T: Component<'t>
{
    fn release_entity(&mut self, entity: usize) {
        if let Some(idx) = self.mapping.remove(&entity) {
            self.storage[idx] = None;
        }
    }
}

impl<'t, T> Default for SpecializedStore<'t, T> 
where
    T: Component<'t>
{
    fn default() -> SpecializedStore<'t, T> {
        SpecializedStore {
            mapping: HashMap::new(),
            storage: Vec::new(),
            _marker: PhantomData
        }
    }
}

#[derive(Default)]
pub struct ComponentStore {
    storage: HashMap<TypeId, Box<dyn Store>>
}

impl ComponentStore {
    pub fn new() -> ComponentStore {
        ComponentStore::default()
    }

    pub fn insert<'t, T>(&mut self, data: T, owner: usize) 
    where
        T: Component<'t>
    {
        // let ty = TypeId::of::<T>();
        // let entry = self.storage.entry(ty)
        //     .or_insert_with(|| Box::new(SpecializedStore::<T>::default()));

        // let downcast: &mut SpecializedStore<T> = entry.as_any_mut().downcast_mut().unwrap();
        // downcast.insert(owner, data);
    }

    pub fn release_entity(&mut self, entity: usize) {
        self.storage.iter_mut().for_each(|(_, v)| v.release_entity(entity));
    }
}