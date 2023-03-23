use std::ops::{Deref, DerefMut};

use crate::bytes::VarInt;

/// Type and size independent vector type
#[derive(Debug, Clone)]
#[repr(C)]
pub struct Vector<T, const N: usize> {
    /// Generically-sized array of components of type `T` and size `N`.
    components: [T; N],
}

impl<T: PartialEq, const N: usize> PartialEq for Vector<T, N> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.components == other.components
    }
}

impl<T: Eq, const N: usize> Eq for Vector<T, N> {}

impl<T: Clone, const N: usize> Vector<T, N> {
    /// Returns the components of this vector by value
    #[inline]
    pub fn components(&self) -> [T; N] {
        self.components.clone()
    }
}

impl<T, const N: usize> Vector<T, N> {
    /// Returns a reference to the components of this vector.
    #[inline]
    pub fn components_ref(&self) -> &[T; N] {
        &self.components
    }

    /// Returns a mutable reference to the components of this vector.
    #[inline]
    pub fn components_mut(&mut self) -> &mut [T; N] {
        &mut self.components
    }
}

impl<T, const N: usize> From<[T; N]> for Vector<T, N> {
    #[inline]
    fn from(components: [T; N]) -> Self {
        Self { components }
    }
}

impl<T> From<(T,)> for Vector<T, 1> {
    #[inline]
    fn from(value: (T,)) -> Self {
        Self { components: [value.0] }
    }
}

impl<T> From<(T, T)> for Vector<T, 2> {
    #[inline]
    fn from(value: (T, T)) -> Self {
        Self { components: [value.0, value.1] }
    }
}

impl<T> From<(T, T, T)> for Vector<T, 3> {
    #[inline]
    fn from(value: (T, T, T)) -> Self {
        Self { components: [value.0, value.1, value.2] }
    }
}

impl<T> From<(T, T, T, T)> for Vector<T, 4> {
    #[inline]
    fn from(value: (T, T, T, T)) -> Self {
        Self { components: [value.0, value.1, value.2, value.3] }
    }
}

/// Maps a 1-vector to directly-accessible fields.
///
/// Internally, [`Vector`] stores its components as an array to allow for generic size.
/// The components can be retrieved from the vector using the [`components_*`](Vector::components)
/// family of methods.
///
/// This however, provides an alternate implementation that provides direct access to fields instead.
/// Under the hood, this is implemented using [`Deref`] and [`DerefMut`] implementations for each
/// vector arity up to 4.
///
/// The compiler initially cannot these fields on the vector and therefore looks at the
/// deref implementations. The deref implementations transmute the vector array to one of these
/// field structs.
///
/// # Example
///
/// ```rust
/// # use pyro_util::Vector;
/// # fn main() { ///
/// let mut vec = Vector::from([0]);
/// vec.x = 1;
///
/// assert_eq!(vec, Vector::from([1]));
/// # }
/// ```
#[repr(C)]
pub struct VectorFields1<T> {
    pub x: T,
}

impl<T> Deref for Vector<T, 1> {
    type Target = VectorFields1<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: `Vector<T, 1>` and `VectorFields1<T>` are guaranteed to have the same
        // layout due to the `repr(C)` attribute.
        // It is therefore safe to cast from one to the other
        unsafe { &*(self as *const _ as *const VectorFields1<T>) }
    }
}

impl<T> DerefMut for Vector<T, 1> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: `Vector<T, 1>` and `VectorFields1<T>` are guaranteed to have the same
        // layout due to the `repr(C)` attribute.
        // It is therefore safe to cast from one to the other
        unsafe { &mut *(self as *mut _ as *mut VectorFields1<T>) }
    }
}

/// Maps a 2-vector to directly-accessible fields.
///
/// Internally, [`Vector`] stores its components as an array to allow for generic size.
/// The components can be retrieved from the vector using the [`components_*`](Vector::components)
/// family of methods.
///
/// This however, provides an alternate implementation that provides direct access to fields instead.
/// Under the hood, this is implemented using [`Deref`] and [`DerefMut`] implementations for each
/// vector arity up to 4.
///
/// The compiler initially cannot these fields on the vector and therefore looks at the
/// deref implementations. The deref implementations transmute the vector array to one of these
/// field structs.
///
/// # Example
///
/// ```rust
/// # use pyro_util::Vector;
/// # fn main() { ///
/// let mut vec = Vector::from([0, 1]);
/// vec.x = 1;
/// vec.y = 2;
///
/// assert_eq!(vec, Vector::from([1, 2]));
/// # }
/// ```
#[repr(C)]
pub struct VectorFields2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Deref for Vector<T, 2> {
    type Target = VectorFields2<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: `Vector<T, 2>` and `VectorFields2<T>` are guaranteed to have the same
        // layout due to the `repr(C)` attribute.
        // It is therefore safe to cast from one to the other
        unsafe { &*(self as *const _ as *const VectorFields2<T>) }
    }
}

impl<T> DerefMut for Vector<T, 2> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: `Vector<T, 2>` and `VectorFields2<T>` are guaranteed to have the same
        // layout due to the `repr(C)` attribute.
        // It is therefore safe to cast from one to the other
        unsafe { &mut *(self as *mut _ as *mut VectorFields2<T>) }
    }
}

/// Maps a 3-vector to directly-accessible fields.
///
/// Internally, [`Vector`] stores its components as an array to allow for generic size.
/// The components can be retrieved from the vector using the [`components_*`](Vector::components)
/// family of methods.
///
/// This however, provides an alternate implementation that provides direct access to fields instead.
/// Under the hood, this is implemented using [`Deref`] and [`DerefMut`] implementations for each
/// vector arity up to 4.
///
/// The compiler initially cannot these fields on the vector and therefore looks at the
/// deref implementations. The deref implementations transmute the vector array to one of these
/// field structs.
///
/// # Example
///
/// ```rust
/// # use pyro_util::Vector;
/// # fn main() { ///
/// let mut vec = Vector::from([0, 1, 2]);
/// vec.x = 1;
/// vec.y = 2;
/// vec.z = 3;
///
/// assert_eq!(vec, Vector::from([1, 2, 3]));
/// # }
/// ```
#[repr(C)]
pub struct VectorFields3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Deref for Vector<T, 3> {
    type Target = VectorFields3<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: `Vector<T, 3>` and `VectorFields3<T>` are guaranteed to have the same
        // layout due to the `repr(C)` attribute.
        // It is therefore safe to cast from one to the other
        unsafe { &*(self as *const _ as *const VectorFields3<T>) }
    }
}

impl<T> DerefMut for Vector<T, 3> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: `Vector<T, 3>` and `VectorFields3<T>` are guaranteed to have the same
        // layout due to the `repr(C)` attribute.
        // It is therefore safe to cast from one to the other
        unsafe { &mut *(self as *mut _ as *mut VectorFields3<T>) }
    }
}

/// Maps a 4-vector to directly-accessible fields.
///
/// Internally, [`Vector`] stores its components as an array to allow for generic size.
/// The components can be retrieved from the vector using the [`components_*`](Vector::components)
/// family of methods.
///
/// This however, provides an alternate implementation that provides direct access to fields instead.
/// Under the hood, this is implemented using [`Deref`] and [`DerefMut`] implementations for each
/// vector arity up to 4.
///
/// The compiler initially cannot these fields on the vector and therefore looks at the
/// deref implementations. The deref implementations transmute the vector array to one of these
/// field structs.
///
/// # Example
///
/// ```rust
/// # use pyro_util::Vector;
/// # fn main() { ///
/// let mut vec = Vector::from([0, 1, 2, 3]);
/// vec.x = 1;
/// vec.y = 2;
/// vec.z = 3;
/// vec.w = 4;
///
/// assert_eq!(vec, Vector::from([1, 2, 3, 4]));
/// # }
/// ```
#[repr(C)]
pub struct VectorFields4<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

impl<T> Deref for Vector<T, 4> {
    type Target = VectorFields4<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: `Vector<T, 4>` and `VectorFields4<T>` are guaranteed to have the same
        // layout due to the `repr(C)` attribute.
        // It is therefore safe to cast from one to the other
        unsafe { &*(self as *const _ as *const VectorFields4<T>) }
    }
}

impl<T> DerefMut for Vector<T, 4> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: `Vector<T, 4>` and `VectorFields4<T>` are guaranteed to have the same
        // layout due to the `repr(C)` attribute.
        // It is therefore safe to cast from one to the other
        unsafe { &mut *(self as *mut _ as *mut VectorFields4<T>) }
    }
}

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
