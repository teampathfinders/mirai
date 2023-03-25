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

pub trait Spawnable<'w> {
    fn store_all(self, owner: usize, store: &mut ComponentStore<'w>);
}

impl<'t, 'w, T> Spawnable<'w> for T 
where 
    T: Component<'t>
{
    fn store_all(self, owner: usize, store: &mut ComponentStore<'w>) {
        store.insert(self, owner);
    }
}

impl<'t, 'w, T0, T1> Spawnable<'w> for (T0, T1) 
where 
    T0: Component<'t>,
    T1: Component<'t>
{
    fn store_all(self, owner: usize, store: &mut ComponentStore<'w>) {
        store.insert(self.0, owner);
        store.insert(self.1, owner);
    }
}

trait Store<'w> {
    // fn as_any_mut(&mut self) -> &mut dyn Any;
    fn release_entity(&mut self, entity: usize);
}

pub struct SpecializedStore<'w, 't, T>
where
    T: Component<'t> + 't
{
    mapping: HashMap<usize, usize>,
    storage: Vec<Option<T>>,
    _marker: PhantomData<(&'w (), &'t ())>
}

impl<'w, 't, T> SpecializedStore<'w, 't, T>
where
    T: Component<'t> + 'w,
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

impl<'w, 't, T> Store<'w> for SpecializedStore<'w, 't, T> 
where
    T: Component<'t> + 'w
{
    fn release_entity(&mut self, entity: usize) {
        if let Some(idx) = self.mapping.remove(&entity) {
            self.storage[idx] = None;
        }
    }
}

impl<'w, 't, T> Default for SpecializedStore<'w, 't, T> 
where
    T: Component<'t>
{
    fn default() -> SpecializedStore<'w, 't, T> {
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

impl<'t, 'w> ComponentStore<'w> {
    pub fn new() -> ComponentStore<'w> {
        ComponentStore::default()
    }

    pub fn insert<T>(&mut self, data: T, owner: usize) 
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

impl<'w> Default for ComponentStore<'w> {
    fn default() -> ComponentStore<'w> {
        ComponentStore {
            storage: HashMap::new()
        }
    }
}