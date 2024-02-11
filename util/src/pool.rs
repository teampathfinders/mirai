use std::{fmt::{self, Debug}, io::Write, ops::{Deref, DerefMut}};
use parking_lot::Mutex;

static BYTE_POOL: Pool<Vec<u8>> = Pool::new();

/// An item that can be used in a global memory pool.
pub trait Poolable: Sized + Default + 'static {
    fn pool() -> &'static Pool<Self>;

    /// Resets the collection, returning it back to the pool.
    fn reset(&mut self);
}

impl Poolable for Vec<u8> {
    #[inline]
    fn pool() -> &'static Pool<Vec<u8>> { 
        &BYTE_POOL 
    }

    #[inline]
    fn reset(&mut self) {
        self.clear();
    }
}

#[derive(Default)]
pub struct Reusable<T: Poolable> {
    inner: T
}

impl<T: Poolable> Reusable<T> {
    /// Returns a collection from the pool or creates a new one if none are available.
    #[inline]
    pub fn alloc() -> Reusable<T> {
        let pool = T::pool();
        pool.alloc()
    }

    /// Returns a collection from the pool or creates a new one using the given closure if none are available.
    pub fn alloc_with<F: FnOnce() -> T>(init: F) -> Reusable<T> {
        let pool = T::pool();
        pool.alloc_with(init)
    }

    /// Returns the inner value.
    /// 
    /// # Warning
    /// After taking the value out of this `Reusable` it will no longer be returned
    /// to the pool automatically. Create a new `Reusable` to put it back into the pool.
    #[inline]
    pub fn into_inner(mut self) -> T {
        let inner = std::mem::take(&mut self.inner);
        std::mem::forget(self);

        inner
    }
}

impl<T: Clone> Reusable<Vec<T>> where Vec<T>: Poolable {
    /// Returns a collection with the given capacity. 
    /// 
    /// If there is a collection available with the given capacity, it will be returned .
    /// If no collections have the requested capacity, a collection from the pool will be resized and returned.
    /// If the pool is empty a new collection will be created with the requested capacity.
    pub fn alloc_with_capacity(cap: usize) -> Reusable<Vec<T>> {
        let pool = <Vec<T>>::pool();
        pool.alloc_with_capacity(cap)
    }

    pub fn alloc_from_slice(slice: &[T]) -> Reusable<Vec<T>> {
        let mut collection: Reusable<Vec<T>> = Reusable::alloc_with_capacity(slice.len());
        collection.extend_from_slice(slice);
        collection
    }
}

impl<T: Clone> From<&[T]> for Reusable<Vec<T>> 
where
    Vec<T>: Poolable
{
    #[inline]
    fn from(value: &[T]) -> Self {
        Reusable::alloc_from_slice(value)
    }
}

impl Write for Reusable<Vec<u8>> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        if self.capacity() - self.len() < buf.len() {
            let deficit = buf.len() - (self.capacity() - self.len());
            tracing::info!("Allocating {deficit} bytes");
        }
        self.inner.write_all(buf)
    }
}

impl AsRef<[u8]> for Reusable<Vec<u8>> {
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

impl AsMut<[u8]> for Reusable<Vec<u8>> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.inner
    }
}

impl<T: Poolable + Debug> Debug for Reusable<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T: Poolable + PartialEq> PartialEq for Reusable<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<T: Poolable + Eq> Eq for Reusable<T> {}

impl<T: Poolable> Drop for Reusable<T> {
    fn drop(&mut self) {
        self.inner.reset();

        let container = std::mem::take(self);
        T::pool().reuse(container.into_inner());
    }
}

impl<T: Poolable + Clone> Clone for Reusable<T> {
    fn clone(&self) -> Reusable<T> {
        Reusable::from(self.inner.clone())
    }
}

impl<T: Poolable> From<T> for Reusable<T> {
    fn from(value: T) -> Self {
        Reusable { inner: value }
    }
}

impl<T: Poolable> Deref for Reusable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Poolable> DerefMut for Reusable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub type BVec = Reusable<Vec<u8>>;

pub struct Pool<T: Poolable> {
    items: Mutex<Vec<T>>
}

impl<T: Poolable> Pool<T> {
    pub const fn new() -> Pool<T> {
        Pool { items: Mutex::new(Vec::new()) }
    }

    #[inline]
    pub fn alloc(&self) -> Reusable<T> {
        let pop = {
            let mut lock = self.items.lock();
            lock.pop()
        };

        Reusable::from(if let Some(value) = pop {
            tracing::debug!("Used vec from pool");
            value
        } else {
            T::default()
        })
    }

    pub fn alloc_with<F: FnOnce() -> T>(&self, init: F) -> Reusable<T> {
        let pop = {
            let mut lock = self.items.lock();
            lock.pop()
        };

        Reusable::from(if let Some(value) = pop {
            tracing::debug!("Used vec from pool");
            value
        } else {
            init()
        })
    }

    #[inline]
    pub fn reuse(&self, value: T) {
        self.items.lock().push(value);
    }
}

impl<T> Pool<Vec<T>> where Vec<T>: Poolable {
    pub fn debug_print(&self) {
        let lock = self.items.lock();
        let mut total = 0;
        for item in lock.iter() {
            let cap = item.capacity();
            total += cap;

            print!("{} ", item.capacity());
        }
        println!("\nTotal size: {total}");
    }

    pub fn alloc_with_capacity(&self, cap: usize) -> Reusable<Vec<T>> {
        // Skip whole capacity procedure when the capacity is 0.
        if cap == 0 {
            return self.alloc();
        }

        let pop = {
            let mut largest_idx = 0;
            let mut largest = 0;
            let mut lock = self.items.lock();

            if lock.is_empty() {
                None
            } else {
                // Find collection with largest capacity
                for (idx, collection) in lock.iter().enumerate() {
                    if collection.capacity() > largest {
                        largest_idx = idx;
                        largest = collection.capacity();
                    }
                }

                Some(lock.swap_remove(largest_idx))
            }   
        };

        Reusable::from(if let Some(mut value) = pop {
            if value.capacity() < cap {
                value.reserve(cap);
                tracing::debug!("Resized vec from pool");
            }
            // tracing::debug!("Reused cap vec from pool");

            value
        } else {
            Vec::with_capacity(cap)
        })
    }
}