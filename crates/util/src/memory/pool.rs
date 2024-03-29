use parking_lot::Mutex;
use std::{
    mem::MaybeUninit,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::Recycled;

static BINARY_POOL: RecyclePool<Vec<u8>> = RecyclePool::new();

// The amount of buffers the `alloc_with_capacity` function will check
// before residing to the largest found. This is to ensure that the function does not
// take an incredibly long time because it is checking all available buffers.
const POOL_MAX_SEARCH_COUNT: usize = 10;

/// A pooled vector.
pub type RVec = Recycled<Vec<u8>>;

/// A pooled string.
///
/// The string uses `Vec<u8>` as a backing storage and therefore shares the pool
/// with [`PVec`].
pub type RString = Recycled<String>;

/// A storage type that can be used by a pool.
pub trait RecycleStorage: Sized + 'static {}

/// Specialization of [`RecycleStorage`] that is only implemented by collections.
///
/// This trait allows [`RecyclePool`] to provide functionality related to collection capacities.
pub trait RecycleCollectionStorage: RecycleStorage {
    /// The capacity of this storage object.
    fn capacity(&self) -> usize;
    /// Reserves additional capacity for the storage object.
    fn reserve(&mut self, capacity: usize);
    /// Creates a new storage object with the given capacity.
    fn with_capacity(capacity: usize) -> Self;
}

impl RecycleStorage for Vec<u8> {}

impl RecycleCollectionStorage for Vec<u8> {
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
pub trait Recyclable: Sized + 'static {
    /// Underlying storage used for this type.
    ///
    /// A `String` can be created from and turned into a `Vec<u8>` which would
    /// therefore be a valid storage type.
    type Storage: RecycleStorage;

    /// Returns the pool associated with this poolable's backing storage.
    fn pool() -> &'static RecyclePool<Self::Storage>;

    /// Converts a storage type into a usable type.
    fn into_usable(storage: Self::Storage) -> Self;

    /// Resets the collection, converting it to its underlying storage
    /// and returning it back to the associated pool.
    fn into_storage(self) -> Self::Storage;
}

impl Recyclable for Vec<u8> {
    type Storage = Vec<u8>;

    #[inline]
    fn pool() -> &'static RecyclePool<Vec<u8>> {
        &BINARY_POOL
    }

    #[inline]
    fn into_usable(storage: Vec<u8>) -> Vec<u8> {
        storage
    }

    #[inline]
    fn into_storage(mut self) -> Vec<u8> {
        self.clear();
        self
    }
}

impl Recyclable for String {
    type Storage = Vec<u8>;

    #[inline]
    fn pool() -> &'static RecyclePool<Vec<u8>> {
        &BINARY_POOL
    }

    #[inline]
    fn into_usable(storage: Vec<u8>) -> Self {
        // This does not panic because `storage` will always be an empty vector
        // and therefore a valid UTF-8 string.
        #[allow(clippy::unwrap_used)]
        String::from_utf8(storage).unwrap()
    }

    #[inline]
    fn into_storage(mut self) -> Self::Storage {
        self.clear();
        self.into_bytes()
    }
}

/// A pool that stores objects of type `S`.
pub struct RecyclePool<S: RecycleStorage> {
    items: Mutex<Vec<S>>,
}

impl<S: RecycleStorage> RecyclePool<S> {
    /// Creates a new pool.
    pub const fn new() -> RecyclePool<S> {
        RecyclePool { items: Mutex::new(Vec::new()) }
    }

    /// Retrieves an object from the pool.
    ///
    /// If the pool had no available objects, a new one is initialised by calling `init`.
    pub fn alloc_with<P, F: FnOnce() -> P>(&self, init: F) -> Recycled<P>
    where
        P: Recyclable<Storage = S>,
    {
        REQ_COUNTER.fetch_add(1, Ordering::Relaxed);

        let pop = {
            let mut lock = self.items.lock();
            lock.pop()
        };

        let vec = pop.map_or_else(
            || {
                ALLOC_COUNTER.fetch_add(1, Ordering::Relaxed);
                init()
            },
            |value| P::into_usable(value),
        );

        Recycled { inner: MaybeUninit::new(vec) }
    }

    /// Takes ownership of the object and returns it to its pool.
    #[inline]
    pub fn recycle(&self, value: S) {
        RECYCLE_COUNTER.fetch_add(1, Ordering::Relaxed);
        self.items.lock().push(value);
    }
}

impl<S: RecycleStorage> RecyclePool<S> {
    /// Retrieves an object from the pool.
    ///
    /// If the pool had no available objects, a new one is initialized using its [`Default`]
    /// implementation.
    #[inline]
    pub fn alloc<P>(&self) -> Recycled<P>
    where
        P: Recyclable<Storage = S> + Default,
    {
        REQ_COUNTER.fetch_add(1, Ordering::Relaxed);

        let pop = {
            let mut lock = self.items.lock();
            lock.pop()
        };

        let vec = pop.map_or_else(
            || {
                ALLOC_COUNTER.fetch_add(1, Ordering::Relaxed);
                P::default()
            },
            |value| P::into_usable(value),
        );

        Recycled { inner: MaybeUninit::new(vec) }
    }
}

impl<T> RecyclePool<T>
where
    T: RecycleCollectionStorage,
{
    /// Retrieves a collection object from the pool with the given capacity.
    ///
    /// This function attempts to find an object with at least the specified capacity.
    /// If none of the searched objects have a big enough capacity, the largest object is taken
    /// and resized to the requested capacity.
    pub fn alloc_with_capacity<P>(&self, cap: usize) -> Recycled<Vec<P>>
    where
        Vec<P>: Recyclable<Storage = T>,
    {
        // Skip whole capacity procedure when the capacity is 0.
        if cap == 0 {
            return self.alloc();
        }

        REQ_COUNTER.fetch_add(1, Ordering::Relaxed);

        let found = {
            let mut largest_idx = 0;
            let mut largest = 0;
            let mut lock = self.items.lock();

            if lock.is_empty() {
                None
            } else {
                // Find collection with largest capacity
                let taken = lock.iter().enumerate().take(POOL_MAX_SEARCH_COUNT);
                for (idx, collection) in taken {
                    if collection.capacity() > cap {
                        largest_idx = idx;
                        break;
                    }

                    if collection.capacity() > largest {
                        largest_idx = idx;
                        largest = collection.capacity();
                    }
                }

                Some(lock.swap_remove(largest_idx))
            }
        };

        let vec = found.map_or_else(
            || {
                ALLOC_COUNTER.fetch_add(1, Ordering::Relaxed);
                T::with_capacity(cap)
            },
            |mut value| {
                if value.capacity() < cap {
                    value.reserve(cap);
                    ALLOC_COUNTER.fetch_add(1, Ordering::Relaxed);
                }

                value
            },
        );

        Recycled {
            inner: MaybeUninit::new(<Vec<P>>::into_usable(vec)),
        }
    }
}

/// Returns the total amount of objects that have been requested from *all* pools.
pub fn total_requests() -> usize {
    REQ_COUNTER.load(Ordering::Relaxed)
}

/// Returns the total amount of objects that have been returned to *all* pools.
pub fn total_recycles() -> usize {
    RECYCLE_COUNTER.load(Ordering::Relaxed)
}

/// Returns the total amount of heap allocations that *all* pools have performed.
pub fn total_allocations() -> usize {
    ALLOC_COUNTER.load(Ordering::Relaxed)
}

pub(super) static REQ_COUNTER: AtomicUsize = AtomicUsize::new(0);
pub(super) static ALLOC_COUNTER: AtomicUsize = AtomicUsize::new(0);
pub(super) static RECYCLE_COUNTER: AtomicUsize = AtomicUsize::new(0);
