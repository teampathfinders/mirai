use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use crate::system::SysParam;

pub trait Resource {

}

pub struct Res<T: Resource> {
    _marker: PhantomData<T>
}

impl<T: Resource> Deref for Res<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        todo!()
    }
}

impl<'a, T: Resource> SysParam<'a> for Res<T> {
    const EXCLUSIVE: bool = false;
}

pub struct ResMut<T: Resource> {
    _marker: PhantomData<T>
}

impl<T: Resource> Deref for ResMut<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        todo!()
    }
}

impl<T: Resource> DerefMut for ResMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        todo!()
    }
}

impl<'a, T: Resource> SysParam<'a> for ResMut<T> {
    const EXCLUSIVE: bool = true;
}