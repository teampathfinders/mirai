use std::{
    ffi::{c_int, c_void},
    ptr::NonNull,
};

use crate::ffi;

/// Combines multiple operations into one large batch.
pub struct WriteBatch {
    pub(crate) ptr: NonNull<c_void>,
}

impl WriteBatch {
    /// Creates a new batch.
    /// This batch can be reused by calling [`clear`](Self::clear) and executed using [`execute`](super::Database::execute).
    pub fn new() -> Self {
        let ptr = unsafe { ffi::batch_new() };
        Self {
            ptr: NonNull::new(ptr).expect("batch pointer is null"),
        }
    }

    /// Adds a put operation to the batch.
    pub fn put<K, V>(&mut self, key: K, val: V)
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        let key = key.as_ref();
        let val = val.as_ref();

        unsafe {
            ffi::batch_put(
                self.ptr.as_ptr(),
                key.as_ptr() as *const i8,
                key.len() as c_int,
                val.as_ptr() as *const i8,
                val.len() as c_int,
            );
        }
    }

    /// Adds a delete operation to the batch.
    pub fn delete<K>(&mut self, key: K)
    where
        K: AsRef<[u8]>,
    {
        let key = key.as_ref();

        unsafe {
            ffi::batch_delete(self.ptr.as_ptr(), key.as_ptr() as *const i8, key.len() as c_int);
        }
    }

    /// Clears the batch, removing all stored operations.
    pub fn clear(&mut self) {
        unsafe {
            ffi::batch_clear(self.ptr.as_ptr());
        }
    }
}

impl Drop for WriteBatch {
    fn drop(&mut self) {
        unsafe {
            ffi::batch_destroy(self.ptr.as_ptr());
        }
    }
}
