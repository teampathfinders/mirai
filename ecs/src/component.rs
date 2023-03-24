use crate::{world::ComponentStore, EntityId};

pub trait Component {

}

impl<T> Component for &T where T: Component {}
impl<T> Component for &mut T where T: Component {}

pub trait Insertable {
    fn store_all(self, owner: EntityId, store: &mut ComponentStore);
}

impl<T> Insertable for T 
where 
    T: Component + 'static 
{
    fn store_all(self, owner: EntityId, store: &mut ComponentStore) {
        store.insert(self, owner);
    }
}

impl<T0, T1> Insertable for (T0, T1) 
where 
    T0: Component + 'static, 
    T1: Component + 'static 
{
    fn store_all(self, owner: EntityId, store: &mut ComponentStore) {
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