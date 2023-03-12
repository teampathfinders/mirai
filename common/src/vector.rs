use std::ops::{Deref, DerefMut};

use crate::VarInt;

/// Type and size independent vector type
#[derive(Debug, Clone)]
#[repr(C)]
pub struct Vector<T, const N: usize> {
    components: [T; N],
}

impl<T: Clone, const N: usize> Vector<T, N> {
    pub fn components(&self) -> [T; N] {
        self.components.clone()
    }
}

impl<T, const N: usize> Vector<T, N> {
    pub fn components_ref(&self) -> &[T; N] {
        &self.components
    }

    pub fn components_mut(&mut self) -> &mut [T; N] {
        &mut self.components
    }
}

impl<T, const N: usize> From<[T; N]> for Vector<T, N> {
    fn from(components: [T; N]) -> Self {
        Self { components }
    }
}

impl<const N: usize> Vector<f32, N> {
    pub fn serialize(&self, buffer: &mut BytesMut) {
        for i in 0..N {
            buffer.put_f32(self.components[i]);
        }
    }
}

#[repr(C)]
pub struct Vector1Accessors<T> {
    pub x: T,
}

impl<T> Deref for Vector<T, 1> {
    type Target = Vector1Accessors<T>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self as *const _ as *const Vector1Accessors<T>) }
    }
}

impl<T> DerefMut for Vector<T, 1> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self as *mut _ as *mut Vector1Accessors<T>) }
    }
}

#[repr(C)]
pub struct Vector2Accessors<T> {
    pub x: T,
    pub y: T,
}

impl<T> Deref for Vector<T, 2> {
    type Target = Vector2Accessors<T>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self as *const _ as *const Vector2Accessors<T>) }
    }
}

impl<T> DerefMut for Vector<T, 2> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self as *mut _ as *mut Vector2Accessors<T>) }
    }
}

#[repr(C)]
pub struct Vector3Accessors<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Deref for Vector<T, 3> {
    type Target = Vector3Accessors<T>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self as *const _ as *const Vector3Accessors<T>) }
    }
}

impl<T> DerefMut for Vector<T, 3> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self as *mut _ as *mut Vector3Accessors<T>) }
    }
}

#[repr(C)]
pub struct Vector4Accessors<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

impl<T> Deref for Vector<T, 4> {
    type Target = Vector4Accessors<T>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self as *const _ as *const Vector4Accessors<T>) }
    }
}

impl<T> DerefMut for Vector<T, 4> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self as *mut _ as *mut Vector4Accessors<T>) }
    }
}

/// 32-bit float vector with 2 components
pub type Vector2f = Vector<f32, 2>;
/// 32-bit float vector with 3 components
pub type Vector3f = Vector<f32, 3>;
/// 32-bit float vector with 4 components
pub type Vector4f = Vector<f32, 4>;
/// 32-bit signed integer vector with 2 components.
pub type Vector2i = Vector<i32, 2>;
/// 32-bit signed integer vector with 3 components.
pub type Vector3i = Vector<i32, 3>;
pub type Vector3b = Vector<u8, 3>;

#[derive(Debug, Clone)]
pub struct BlockPosition {
    pub x: i32,
    pub y: u32,
    pub z: i32,
}

impl BlockPosition {
    pub const fn new(x: i32, y: u32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn serialized_size(&self) -> usize {
        self.x.var_len() + self.y.var_len() + self.z.var_len()
    }
}
