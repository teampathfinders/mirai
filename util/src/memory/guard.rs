use std::{fmt::{self, Debug}, io::Write, mem::MaybeUninit, ops::{Deref, DerefMut}, sync::atomic::Ordering};

use crate::{PoolCollectionStorage, Poolable};
use super::ALLOC_COUNTER;

/// Wrapper around an object that automatically returns it to its pool when dropped.
#[repr(transparent)]
pub struct Pooled<T: Poolable> {
    pub(super) inner: MaybeUninit<T>
}

impl<T: Poolable> Pooled<T> {
    /// Returns a collection from the pool or creates a new one using the given closure if none are available.
    pub fn alloc_with<F: FnOnce() -> T>(init: F) -> Pooled<T> {
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

impl<T: Poolable + Default> Pooled<T> {
    /// Returns a collection from the pool or creates a new one if none are available.
    #[inline]
    pub fn alloc() -> Pooled<T> {
        let pool = T::pool();
        pool.alloc()
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/// STRING GUARD IMPLEMENTATIONS                                                                                   ///
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl From<&str> for Pooled<String> {
    fn from(value: &str) -> Pooled<String> {
        let bin = Pooled::alloc_from_slice(value.as_bytes());
        let inner = bin.into_inner();

        Pooled::from(String::from_utf8(inner).unwrap())
    }
}

impl Clone for Pooled<String> {
    fn clone(&self) -> Pooled<String> {
        Pooled::from(self.as_str())
    }
}

impl Default for Pooled<String> {
    #[inline]
    fn default() -> Self {
        Pooled::alloc()
    }
}

impl serde::Serialize for Pooled<String> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer 
    {
        self.deref().serialize(serializer)
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/// VECTOR GUARD IMPLEMENTATIONS                                                                                   ///
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl<T: Clone> Pooled<Vec<T>> 
where 
    Vec<T>: Poolable, 
    <Vec<T> as Poolable>::Storage: PoolCollectionStorage  
{
    /// Returns a collection with the given capacity. 
    /// 
    /// If there is a collection available with the given capacity, it will be returned .
    /// If no collections have the requested capacity, a collection from the pool will be resized and returned.
    /// If the pool is empty, a new collection will be created with the requested capacity.
    pub fn alloc_with_capacity(cap: usize) -> Pooled<Vec<T>> {
        let pool = <Vec<T>>::pool();
        pool.alloc_with_capacity(cap)
    }

    /// Returns a collection with the given data.
    /// 
    /// This function will ensure that the collection has enough capacity to fit 
    /// the data and will then copy it to the collection.
    /// 
    /// See [`alloc_with_capacity`](Reusable::alloc_with_capacity) for details on allocation.
    pub fn alloc_from_slice(slice: &[T]) -> Pooled<Vec<T>> {
        let mut collection: Pooled<Vec<T>> = Pooled::alloc_with_capacity(slice.len());

        if collection.capacity() < slice.len() {
            ALLOC_COUNTER.fetch_add(1, Ordering::Relaxed);
        }

        collection.extend_from_slice(slice);
        collection
    }
}

impl<T: Clone> From<&[T]> for Pooled<Vec<T>> 
where
    Vec<T>: Poolable, 
    <Vec<T> as Poolable>::Storage: PoolCollectionStorage
{
    #[inline]
    fn from(value: &[T]) -> Self {
        Pooled::alloc_from_slice(value)
    }
}

impl Write for Pooled<Vec<u8>> {
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

impl AsRef<[u8]> for Pooled<Vec<u8>> {
    fn as_ref(&self) -> &[u8] {
        // SAFETY: This is safe because `inner` will always be initialised except when it
        // is being dropped. Since calling this function means the object is still referenced, it is
        // initialized.
        unsafe { self.inner.assume_init_ref() }
    }
}

impl AsMut<[u8]> for Pooled<Vec<u8>> {
    fn as_mut(&mut self) -> &mut [u8] {
        // SAFETY: This is safe because `inner` will always be initialised except when it
        // is being dropped. Since calling this function means the object is still referenced, it is
        // initialized.
        unsafe { self.inner.assume_init_mut() }
    }
}

impl<T: Clone> Clone for Pooled<Vec<T>> 
where 
    Vec<T>: Poolable, 
    <Vec<T> as Poolable>::Storage: PoolCollectionStorage 
{
    fn clone(&self) -> Pooled<Vec<T>> {
        // SAFETY: This is safe because `inner` will always be initialised except when it
        // is being dropped. Since calling this function means the object is still referenced, it is
        // initialized.
        Pooled::alloc_from_slice(unsafe { self.inner.assume_init_ref() })
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/// GENERAL GUARD IMPLEMENTATIONS                                                                                  ///
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl<T: Poolable + Debug> Debug for Pooled<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T: Poolable + PartialEq> PartialEq for Pooled<T> {
    fn eq(&self, other: &Self) -> bool {
        // SAFETY: This is safe because `inner` will always be initialised except when it
        // is being dropped. Since calling this function means the object is still referenced, it is
        // initialized.
        unsafe { self.inner.assume_init_ref() }.eq(unsafe { other.inner.assume_init_ref() })
    }
}

impl<T: Poolable + Eq> Eq for Pooled<T> {}

impl<T: Poolable> Drop for Pooled<T> {
    fn drop(&mut self) {
        // SAFETY: This is safe because `inner` will always be initialised except when it
        // is being dropped. Since calling this function means the object is still referenced, it is
        // initialized.
        let inner = unsafe {
            self.inner.assume_init_read()
        };

        let inner = inner.into_storage();
        T::pool().dealloc(inner)
    }
}

impl<T: Poolable> From<T> for Pooled<T> {
    fn from(value: T) -> Self {
        Pooled { inner: MaybeUninit::new(value) }
    }
}

impl<T: Poolable> Deref for Pooled<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: This is safe because `inner` will always be initialised except when it
        // is being dropped. Since calling this function means the object is still referenced, it is
        // initialized.
        unsafe { self.inner.assume_init_ref() }
    }
}

impl<T: Poolable> DerefMut for Pooled<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: This is safe because `inner` will always be initialised except when it
        // is being dropped. Since calling this function means the object is still referenced, it is
        // initialized.
        unsafe { self.inner.assume_init_mut() }
    }
}