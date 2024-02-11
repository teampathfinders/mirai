use std::{io::Write, ops::{Deref, DerefMut}};
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
        std::mem::take(&mut self.inner)
    }
}

impl Write for Reusable<Vec<u8>> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write_all(buf)?;
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

impl<T: Poolable> Drop for Reusable<T> {
    fn drop(&mut self) {
        self.inner.reset();

        let container = std::mem::take(self);
        T::pool().reuse(container.into_inner())
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