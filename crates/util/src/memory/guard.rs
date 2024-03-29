use std::{fmt::{self, Debug}, io::Write, mem::MaybeUninit, ops::{Deref, DerefMut}, sync::atomic::Ordering};

use crate::{RecycleCollectionStorage, Recyclable};
use super::ALLOC_COUNTER;

/// Wrapper around an object that automatically returns it to its pool when dropped.
#[repr(transparent)]
pub struct Recycled<T: Recyclable> {
    pub(super) inner: MaybeUninit<T>
}

impl<T: Recyclable> Recycled<T> {
    /// Returns a collection from the pool or creates a new one using the given closure if none are available.
    pub fn alloc_with<F: FnOnce() -> T>(init: F) -> Recycled<T> {
        let pool = T::pool();
        pool.alloc_with(init)
    }

    /// Returns the inner value.
    /// 
    /// # Warning
    /// After taking the value out of this `Reusable` it will no longer be returned
    /// to the pool automatically. Create a new `Reusable` to put it back into the pool.
    #[inline]
    pub fn into_inner(self) -> T {
        // SAFETY: This is safe because `inner` will always be initialised except when it
        // is being dropped. Since calling this function means the object is still referenced, it is
        // initialized.
        let inner = unsafe {
            self.inner.assume_init_read()
        };

        std::mem::forget(self);
        inner
    }

    /// Destroys the collection.
    fn prune(mut self) {
        // SAFETY: This is safe because `inner` will always be initialised except when it
        // is being dropped. Since calling this function means the object is still referenced, it is
        // initialized.
        unsafe {
            self.inner.assume_init_drop()
        }

        // Don't add the reusable back to the pool.
        std::mem::forget(self);
    }
}

impl<T: Recyclable + Default> Recycled<T> {
    /// Returns a collection from the pool or creates a new one if none are available.
    #[inline]
    pub fn alloc() -> Recycled<T> {
        let pool = T::pool();
        pool.alloc()
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// STRING GUARD IMPLEMENTATIONS                                                                                     //
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[allow(clippy::fallible_impl_from)]
impl From<&str> for Recycled<String> {
    fn from(value: &str) -> Recycled<String> {
        let bin = Recycled::alloc_from_slice(value.as_bytes());
        let inner = bin.into_inner();

        // This does not panic because `inner` is a vector created directly
        // from the bytes of a valid string slice `value`. Therefore
        // it is a valid UTF-8 vector.
        #[allow(clippy::unwrap_used)]
        Recycled::from(String::from_utf8(inner).unwrap())
    }
}

impl Clone for Recycled<String> {
    fn clone(&self) -> Recycled<String> {
        Recycled::from(self.as_str())
    }
}

impl Default for Recycled<String> {
    #[inline]
    fn default() -> Self {
        Recycled::alloc()
    }
}

impl serde::Serialize for Recycled<String> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer 
    {
        self.deref().serialize(serializer)
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// VECTOR GUARD IMPLEMENTATIONS                                                                                     //
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl<T: Clone> Recycled<Vec<T>> 
where 
    Vec<T>: Recyclable, 
    <Vec<T> as Recyclable>::Storage: RecycleCollectionStorage  
{
    /// Returns a collection with the given capacity. 
    /// 
    /// If there is a collection available with the given capacity, it will be returned .
    /// If no collections have the requested capacity, a collection from the pool will be resized and returned.
    /// If the pool is empty, a new collection will be created with the requested capacity.
    pub fn alloc_with_capacity(cap: usize) -> Recycled<Vec<T>> {
        let pool = <Vec<T>>::pool();
        pool.alloc_with_capacity(cap)
    }

    /// Returns a collection with the given data.
    /// 
    /// This function will ensure that the collection has enough capacity to fit 
    /// the data and will then copy it to the collection.
    /// 
    /// See [`alloc_with_capacity`](Reusable::alloc_with_capacity) for details on allocation.
    pub fn alloc_from_slice(slice: &[T]) -> Recycled<Vec<T>> {
        let mut collection: Recycled<Vec<T>> = Recycled::alloc_with_capacity(slice.len());

        if collection.capacity() < slice.len() {
            ALLOC_COUNTER.fetch_add(1, Ordering::Relaxed);
        }

        collection.extend_from_slice(slice);
        collection
    }
}

impl<T: Clone> From<&[T]> for Recycled<Vec<T>> 
where
    Vec<T>: Recyclable, 
    <Vec<T> as Recyclable>::Storage: RecycleCollectionStorage
{
    #[inline]
    fn from(value: &[T]) -> Self {
        Recycled::alloc_from_slice(value)
    }
}

impl Write for Recycled<Vec<u8>> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // SAFETY: This is safe because `inner` will always be initialised except when it
        // is being dropped. Since calling this function means the object is still referenced, it is
        // initialized.
        unsafe { self.inner.assume_init_mut() }.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        // SAFETY: This is safe because `inner` will always be initialised except when it
        // is being dropped. Since calling this function means the object is still referenced, it is
        // initialized.
        unsafe { self.inner.assume_init_mut() }.write_all(buf)
    }
}

impl AsRef<[u8]> for Recycled<Vec<u8>> {
    fn as_ref(&self) -> &[u8] {
        // SAFETY: This is safe because `inner` will always be initialised except when it
        // is being dropped. Since calling this function means the object is still referenced, it is
        // initialized.
        unsafe { self.inner.assume_init_ref() }
    }
}

impl AsMut<[u8]> for Recycled<Vec<u8>> {
    fn as_mut(&mut self) -> &mut [u8] {
        // SAFETY: This is safe because `inner` will always be initialised except when it
        // is being dropped. Since calling this function means the object is still referenced, it is
        // initialized.
        unsafe { self.inner.assume_init_mut() }
    }
}

impl<T: Clone> Clone for Recycled<Vec<T>> 
where 
    Vec<T>: Recyclable, 
    <Vec<T> as Recyclable>::Storage: RecycleCollectionStorage 
{
    fn clone(&self) -> Recycled<Vec<T>> {
        // SAFETY: This is safe because `inner` will always be initialised except when it
        // is being dropped. Since calling this function means the object is still referenced, it is
        // initialized.
        Recycled::alloc_from_slice(unsafe { self.inner.assume_init_ref() })
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// GENERAL GUARD IMPLEMENTATIONS                                                                                    //
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl<T: Recyclable + Debug> Debug for Recycled<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

#[allow(clippy::unconditional_recursion)] // False positive.
impl<T: Recyclable + PartialEq> PartialEq for Recycled<T> {
    fn eq(&self, other: &Self) -> bool {
        // SAFETY: This is safe because `inner` will always be initialised except when it
        // is being dropped. Since calling this function means the object is still referenced, it is
        // initialized.
        let this = unsafe { self.inner.assume_init_ref() };

        // SAFETY: This is safe because `inner` will always be initialised except when it
        // is being dropped. Since calling this function means the object is still referenced, it is
        // initialized.
        let other = unsafe { other.inner.assume_init_ref() };

        this.eq(other)
    }
}

impl<T: Recyclable + Eq> Eq for Recycled<T> {}

impl<T: Recyclable> Drop for Recycled<T> {
    fn drop(&mut self) {
        // SAFETY: This is safe because `inner` will always be initialised except when it
        // is being dropped. Since calling this function means the object is still referenced, it is
        // initialized.
        let inner = unsafe {
            self.inner.assume_init_read()
        };

        let inner = inner.into_storage();
        T::pool().recycle(inner)
    }
}

impl<T: Recyclable> From<T> for Recycled<T> {
    fn from(value: T) -> Self {
        Recycled { inner: MaybeUninit::new(value) }
    }
}

impl<T: Recyclable> Deref for Recycled<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: This is safe because `inner` will always be initialised except when it
        // is being dropped. Since calling this function means the object is still referenced, it is
        // initialized.
        unsafe { self.inner.assume_init_ref() }
    }
}

impl<T: Recyclable> DerefMut for Recycled<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: This is safe because `inner` will always be initialised except when it
        // is being dropped. Since calling this function means the object is still referenced, it is
        // initialized.
        unsafe { self.inner.assume_init_mut() }
    }
}