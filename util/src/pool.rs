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
    #[inline]
    pub fn alloc() -> Reusable<T> {
        let pool = T::pool();
        pool.alloc()
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

        tracing::debug!("Reusable returned to pool");
    }
}

impl<T: Poolable + Clone> Clone for Reusable<T> {
    fn clone(&self) -> Reusable<T> {
        Reusable::from(self.inner.clone())
    }
}

impl<T: Poolable> From<T> for Reusable<T> {
    fn from(value: T) -> Self {
        tracing::debug!("Reusable::from called");
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
            value
        } else {
            T::default()
        })
    }

    #[inline]
    pub fn reuse(&self, value: T) {
        self.items.lock().push(value);
    }
}