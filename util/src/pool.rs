use std::{fmt::{self, Debug}, io::Write, mem::MaybeUninit, ops::{Deref, DerefMut}, sync::atomic::{AtomicUsize, Ordering}};
use parking_lot::Mutex;

static BYTE_POOL: Pool<Vec<u8>> = Pool::new();

// The amount of buffers the `alloc_with_capacity` function will check
// before residing to the largest found. This is to ensure that the function does not
// take an incredibly long time because it is checking all available buffers.
const POOL_MAX_SEARCH_COUNT: usize = 10;

pub trait PoolStorage: Sized + 'static {}

pub trait PoolCollectionStorage: PoolStorage {
    fn capacity(&self) -> usize;
    fn reserve(&mut self, capacity: usize);
    fn with_capacity(capacity: usize) -> Self;
}

impl PoolStorage for Vec<u8> {}

impl PoolCollectionStorage for Vec<u8> {
    fn capacity(&self) -> usize {
        self.capacity()
    }

    fn reserve(&mut self, capacity: usize) {
        self.reserve(capacity);
    }

    fn with_capacity(capacity: usize) -> Self {
        Vec::with_capacity(capacity)
    }
}

/// An item that can be used in a global memory pool.
pub trait Poolable: Sized + 'static {
    /// Underlying storage used for this type.
    /// 
    /// A `String` can be created from and turned into a `Vec<u8>` which would
    /// therefore be a valid storage type.
    type Storage: PoolStorage;

    fn pool() -> &'static Pool<Self::Storage>;

    /// Converts a storage type into a usable type.
    fn into_usable(storage: Self::Storage) -> Self;

    /// Resets the collection, converting it to its underlying storage
    /// and returning it back to the associated pool.
    fn into_storage(self) -> Self::Storage;
}

impl Poolable for Vec<u8> {
    type Storage = Vec<u8>;

    #[inline]
    fn pool() -> &'static Pool<Vec<u8>> { 
        &BYTE_POOL 
    }

    #[inline]
    fn into_usable(storage: Vec<u8>) -> Vec<u8>{ storage }

    #[inline]
    fn into_storage(mut self) -> Vec<u8> {
        self.clear();
        self
    }
}

impl Poolable for String {
    type Storage = Vec<u8>;

    #[inline]
    fn pool() -> &'static Pool<Vec<u8>> {
        &BYTE_POOL
    }

    #[inline]
    fn into_usable(storage: Vec<u8>) -> Self {
        String::from_utf8(storage).unwrap()
    }

    #[inline]
    fn into_storage(mut self) -> Self::Storage {
        self.clear();
        self.into_bytes()
    }
}

#[repr(transparent)]
pub struct Reusable<T: Poolable> {
    inner: MaybeUninit<T>
}

impl<T: Poolable> Reusable<T> {
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
    pub fn into_inner(self) -> T {
        unsafe {
            self.inner.assume_init_read()
        }
    }

    /// Destroys the collection.
    fn prune(mut self) {
        unsafe {
            self.inner.assume_init_drop()
        }

        // Don't add the reusable back to the pool.
        std::mem::forget(self);
    }
}

impl<T: Poolable + Default> Reusable<T> {
    /// Returns a collection from the pool or creates a new one if none are available.
    #[inline]
    pub fn alloc() -> Reusable<T> {
        let pool = T::pool();
        pool.alloc()
    }
}

impl<T: Clone> Reusable<Vec<T>> 
where 
    Vec<T>: Poolable, 
    <Vec<T> as Poolable>::Storage: PoolCollectionStorage  
{
    /// Returns a collection with the given capacity. 
    /// 
    /// If there is a collection available with the given capacity, it will be returned .
    /// If no collections have the requested capacity, a collection from the pool will be resized and returned.
    /// If the pool is empty, a new collection will be created with the requested capacity.
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
    Vec<T>: Poolable, 
    <Vec<T> as Poolable>::Storage: PoolCollectionStorage
{
    #[inline]
    fn from(value: &[T]) -> Self {
        Reusable::alloc_from_slice(value)
    }
}

impl Write for Reusable<Vec<u8>> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        unsafe { self.inner.assume_init_mut() }.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        unsafe { self.inner.assume_init_mut() }.write_all(buf)
    }
}

impl AsRef<[u8]> for Reusable<Vec<u8>> {
    fn as_ref(&self) -> &[u8] {
        unsafe { self.inner.assume_init_ref() }
    }
}

impl AsMut<[u8]> for Reusable<Vec<u8>> {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe { self.inner.assume_init_mut() }
    }
}

impl<T: Poolable + Debug> Debug for Reusable<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T: Poolable + PartialEq> PartialEq for Reusable<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.inner.assume_init_ref() }.eq(unsafe { other.inner.assume_init_ref() })
    }
}

impl<T: Poolable + Eq> Eq for Reusable<T> {}

impl<T: Poolable> Drop for Reusable<T> {
    fn drop(&mut self) {
        let inner = unsafe {
            self.inner.assume_init_read()
        };

        let inner = inner.into_storage();
        T::pool().dealloc(inner)
    }
}

impl<T: Clone> Clone for Reusable<Vec<T>> 
where 
    Vec<T>: Poolable, 
    <Vec<T> as Poolable>::Storage: PoolCollectionStorage 
{
    fn clone(&self) -> Reusable<Vec<T>> {
        Reusable::alloc_from_slice(unsafe { self.inner.assume_init_ref() })
    }
}

impl<T: Poolable> From<T> for Reusable<T> {
    fn from(value: T) -> Self {
        Reusable { inner: MaybeUninit::new(value) }
    }
}

impl<T: Poolable> Deref for Reusable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.assume_init_ref() }
    }
}

impl<T: Poolable> DerefMut for Reusable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.inner.assume_init_mut() }
    }
}

pub type BVec = Reusable<Vec<u8>>;

pub struct Pool<S: PoolStorage> {
    items: Mutex<Vec<S>>
}

impl<S: PoolStorage> Pool<S> {
    pub const fn new() -> Pool<S> {
        Pool { items: Mutex::new(Vec::new()) }
    }

    pub fn alloc_with<P, F: FnOnce() -> P>(&self, init: F) -> Reusable<P> 
    where
        P: Poolable<Storage = S>
    {
        let pop = {
            let mut lock = self.items.lock();
            lock.pop()
        };

        ALLOC_COUNTER.fetch_add(1, Ordering::SeqCst);

        let vec = if let Some(value) = pop {
            P::into_usable(value)
        } else {
            init()
        };

        Reusable { inner: MaybeUninit::new(vec) }
    }

    #[inline]
    pub fn dealloc(&self, value: S) {
        self.items.lock().push(value);
    }
}

impl<S: PoolStorage> Pool<S> {
    #[inline]
    pub fn alloc<P>(&self) -> Reusable<P> 
    where
        P: Poolable<Storage = S> + Default
    {
        let pop = {
            let mut lock = self.items.lock();
            lock.pop()
        };

        ALLOC_COUNTER.fetch_add(1, Ordering::SeqCst);

        let vec = if let Some(value) = pop {
            P::into_usable(value)
        } else {
            P::default()
        };

        Reusable { inner: MaybeUninit::new(vec) }
    }
}

impl<T> Pool<T> where T: PoolCollectionStorage {
    pub fn debug_print(&self) {
        let lock = self.items.lock();
        let mut total = 0;
        for item in lock.iter() {
            let cap = item.capacity();
            total += cap;

            print!("{} ", item.capacity());
        }
        println!("\nTotal size: {total} | Total count: {} | Alloc: {} | Dealloc: {}", lock.len(), ALLOC_COUNTER.load(Ordering::SeqCst), DEALLOC_COUNTER.load(Ordering::SeqCst));
    }

    pub fn alloc_with_capacity<P>(&self, cap: usize) -> Reusable<Vec<P>> 
    where
        Vec<P>: Poolable<Storage = T>
    {
        // Skip whole capacity procedure when the capacity is 0.
        if cap == 0 {
            return self.alloc()
        }

        ALLOC_COUNTER.fetch_add(1, Ordering::SeqCst);

        let found = {
            let mut largest_idx = 0;
            let mut largest = 0;
            let mut lock = self.items.lock();
            
            if lock.is_empty() {
                None
            } else {
                // Find collection with largest capacity
                for (idx, collection) in lock.iter().enumerate().take(POOL_MAX_SEARCH_COUNT) {
                    if collection.capacity() > cap {
                        largest_idx = idx;
                        break    
                    }

                    if collection.capacity() > largest {
                        largest_idx = idx;
                        largest = collection.capacity();
                    }
                }

                Some(lock.swap_remove(largest_idx))
            }   
        };

        let vec = if let Some(mut value) = found {
            if value.capacity() < cap {
                value.reserve(cap);
            }

            value
        } else {
            T::with_capacity(cap)
            // <Vec<T> as PoolCollectionStorage>::with_capacity(cap)
        };

        Reusable { inner: MaybeUninit::new(<Vec<P>>::into_usable(vec)) }
    }
}

static ALLOC_COUNTER: AtomicUsize = AtomicUsize::new(0);
static DEALLOC_COUNTER: AtomicUsize = AtomicUsize::new(0);