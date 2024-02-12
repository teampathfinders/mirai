use anyhow::anyhow;
use util::BVec;

use std::ptr::NonNull;
use std::{
    ffi::{c_void, CStr, CString},
    marker::PhantomData,
    ops::Deref,
    os::raw::{c_char, c_int},
};

use crate::ffi::LoadStatus;
use crate::{ffi, DataKey};

/// Wraps a LevelDB buffer, ensuring the buffer is deallocated after use.
#[derive(Debug)]
pub struct Guard<'a>(&'a mut [u8]);

impl<'a> Guard<'a> {
    /// Creates a `Guard` from the given slice.
    ///
    /// This is not implemented as a `From` trait so that `Guard` can only be constructed
    /// inside of this crate.
    ///
    /// # Safety
    ///
    /// A `Guard` must only be created from a slice that was allocated by
    /// `LevelDb` or `Keys`.
    /// The caller must also ensure that the slice is not referenced anywhere else in the program.
    #[inline]
    pub(crate) unsafe fn from_slice(slice: &'a mut [u8]) -> Self {
        Guard(slice)
    }
}

impl<'a> Deref for Guard<'a> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> AsRef<[u8]> for Guard<'a> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}

impl<'a> Drop for Guard<'a> {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: The slice in self should have been allocated by the database.
        // It is safe to delete because the pointer is unique and guaranteed to exist.
        unsafe {
            ffi::level_deallocate_array(self.0.as_mut_ptr() as *mut i8);
        }
    }
}

/// Reference to a key-value pair returned by the [`Keys`] iterator.
pub struct KvRef<'a> {
    iter: NonNull<c_void>,
    /// Ensures that [`KvRef`] does not outlive the parent [`Keys`] iterator.
    _marker: PhantomData<&'a ()>,
}

impl<'a> KvRef<'a> {
    /// The key associated with this pair.
    #[allow(clippy::missing_panics_doc)] // Panic should never happen.
    pub fn key(&self) -> Guard {
        // SAFETY: A Ref should only exist while the iterator is valid.
        // This invariant is upheld by the lifetime 'a.
        unsafe {
            // `level_iter_key` does not fail.
            let result = ffi::level_iter_key(self.iter.as_ptr());
            assert!(!result.data.is_null(), "Iterator key pointer was null"); // Something is very wrong if this null...

            let slice = std::slice::from_raw_parts_mut(result.data as *mut u8, result.size as usize);
            Guard::from_slice(slice)
        }
    }

    /// The data associated with this pair.
    #[allow(clippy::missing_panics_doc)] // Panic should never happen.
    pub fn value(&self) -> Guard {
        // SAFETY: A Ref should only exist while the iterator is valid.
        // This invariant is upheld by the lifetime 'a.
        unsafe {
            // `level_iter_value` does not fail.
            let result = ffi::level_iter_value(self.iter.as_ptr());
            assert!(!result.data.is_null(), "Iterator value pointer was null"); // Something is very wrong if this null...

            let slice = std::slice::from_raw_parts_mut(result.data as *mut u8, result.size as usize);
            Guard::from_slice(slice)
        }
    }
}

/// Iterator over keys in a LevelDB database.
pub struct Keys<'a> {
    /// Current position of the iterator.
    index: usize,
    /// Pointer to the C++ iterator.
    iter: NonNull<c_void>,
    /// Ensures the iterator does not outlive the database.
    _marker: PhantomData<&'a ()>,
}

impl<'a> Keys<'a> {
    /// Creates a new iterator for the given database.
    #[allow(clippy::missing_panics_doc)] // Panic should never happen.
    pub fn new(db: &'a Database) -> Keys<'a> {
        // SAFETY: level_iter is guaranteed to not return an error.
        // The iterator position has also been initialized by FFI and is not in an invalid state.
        let result = unsafe { ffi::level_iter(db.ptr.as_ptr()) };
        assert!(!result.data.is_null(), "Iterator pointer was null"); // Something is very wrong if this null...

        Keys {
            index: 0,
            // SAFETY: level_iter is guaranteed to not return an error.
            iter: unsafe { NonNull::new_unchecked(result.data) },
            _marker: PhantomData,
        }
    }
}

impl<'a> Iterator for Keys<'a> {
    type Item = KvRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index != 0 {
            // SAFETY: `level_iter_next` is safe to call, as long as the iterator has not been destroyed.
            // * The only way to delete the iterator is using the `Drop` implementation of `Self`,
            // it is therefore safe to call this function.
            unsafe { ffi::level_iter_next(self.iter.as_ptr()) };
        }
        self.index += 1;

        // SAFETY: `level_iter_valid` is safe to call, as long as the iterator has not been destroyed.
        // The only code able to destroy the iter is the `Drop` implementation of `Self`, it is
        // therefore safe to call this function.
        // Furthermore, the above check ensures the iterator is valid and supports the key and value methods
        // provided by `Ref`.
        let valid = unsafe { ffi::level_iter_valid(self.iter.as_ptr()) };
        valid.then_some(KvRef { iter: self.iter, _marker: PhantomData })
    }
}

impl<'a> Drop for Keys<'a> {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: `self` is the only object that is able to modify the iterator.
        // Therefore it is safe to delete because it hasn't been modified externally.
        unsafe {
            ffi::level_destroy_iter(self.iter.as_ptr());
        }
    }
}

/// A LevelDB database.
pub struct Database {
    /// Pointer to the C++ Database struct, containing the database and corresponding options.
    /// This data is heap-allocated and must therefore also be deallocated by C++ when it is no longer needed.
    ptr: NonNull<c_void>,
}

impl Database {
    /// Opens the database at the specified path.
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure that the given `path` is not
    /// already in use by another `Database`.
    /// Multiple databases owning the same directory is *guaranteed* to cause corruption.
    ///
    /// # Errors
    ///
    /// This method returns an error if the database could not be opened.
    pub unsafe fn open<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<str>,
    {
        // Ensure that there can only be a single database open at once.
        // Multiple databases causes corruption.
        let ffi_path = CString::new(path.as_ref())?;

        // SAFETY: This function is guaranteed to not return exceptions.
        // It also does not modify the argument and returns a valid struct.
        unsafe {
            let result = ffi::level_open(ffi_path.as_ptr());
            if result.status == LoadStatus::Success {
                if result.data.is_null() {
                    tracing::error!("Received database was a null pointer despite the result being marked successful");
                    anyhow::bail!("Received database was a null pointer");
                }

                // SAFETY: If result.is_success is true, then the caller has set data to a valid pointer.
                Ok(Self { ptr: NonNull::new_unchecked(result.data) })
            } else {
                Err(translate_ffi_error(result))
            }
        }
    }

    /// Creates a new [`Keys`] iterator.
    #[inline]
    pub fn iter(&self) -> Keys {
        Keys::new(self)
    }

    /// Loads the specified value from the database.
    pub fn get(&self, key: DataKey) -> anyhow::Result<Option<Guard>> {
        let mut raw_key = BVec::alloc_with_capacity(key.serialized_size());
        key.serialize(&mut raw_key)?;

        // SAFETY: This function is guaranteed to not modify any arguments.
        // It also does not throw exceptions and returns a valid struct.
        //
        // A LevelDB database is thread-safe, this function can be used by multiple threads.
        unsafe {
            let result = ffi::level_get(self.ptr.as_ptr(), raw_key.as_ptr() as *const c_char, raw_key.len() as c_int);
            if result.status == LoadStatus::Success {
                if result.data.is_null() {
                    tracing::error!("Received world data is a null pointer despite being marked as a successful result");
                    anyhow::bail!("Received world data is a null pointer");
                }

                // SAFETY: result.data is guaranteed by the caller to be a valid pointer.
                // result.size is also guaranteed to be the size of the actual array.
                let data = std::slice::from_raw_parts_mut(result.data as *mut u8, result.size as usize);

                // SAFETY: The data passed into the Guard has been allocated in the leveldb FFI code.
                // It is therefore also required to deallocate the data there, which is what Guard
                // does.
                Ok(Some(Guard::from_slice(data)))
            } else if result.status == LoadStatus::NotFound {
                Ok(None)
            } else {
                Err(translate_ffi_error(result))
            }
        }
    }

    /// Inserts a new value into the database.
    ///
    /// # Arguments
    /// * `key` - Key to store the value at.
    /// * `value` - Value to store at the specified key.
    pub fn insert<V>(&self, key: DataKey, value: V) -> anyhow::Result<()>
    where
        V: AsRef<[u8]>,
    {
        let mut raw_key = BVec::alloc_with_capacity(key.serialized_size());
        key.serialize(&mut raw_key)?;

        let value = value.as_ref();

        // SAFETY: This is safe because the data and lengths come from properly allocated vecs.
        // Additionally, the insert method does not keep references to the data after the function has been called.
        unsafe {
            let result = ffi::level_insert(
                self.ptr.as_ptr(),
                raw_key.as_ptr() as *const c_char,
                raw_key.len() as c_int,
                value.as_ptr() as *const c_char,
                value.len() as c_int,
            );

            if result.status == LoadStatus::Success {
                Ok(())
            } else {
                Err(translate_ffi_error(result))
            }
        }
    }

    /// Removes the given key from the database.
    pub fn remove(&self, key: DataKey) -> anyhow::Result<()> {
        let mut raw_key = BVec::alloc_with_capacity(key.serialized_size());
        key.serialize(&mut raw_key)?;

        // SAFETY: This is safe because the data and lengths come from properly allocated vecs.
        // Additionally, the remove method does not keep references to the data after the function has been called.
        unsafe {
            let result = ffi::level_remove(self.ptr.as_ptr(), raw_key.as_mut_ptr() as *mut c_char, raw_key.len() as c_int);

            if result.status == LoadStatus::Success || result.status == LoadStatus::NotFound {
                Ok(())
            } else {
                Err(translate_ffi_error(result))
            }
        }
    }
}

impl Drop for Database {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: Make sure to clean up the LevelDB resources when the database is dropped.
        // This can only be done by C++.
        unsafe {
            ffi::level_close(self.ptr.as_ptr());
        }
    }
}

// SAFETY: All LevelDB operations are thread-safe.
unsafe impl Send for Database {}

// SAFETY: All LevelDB operations are thread-safe.
unsafe impl Sync for Database {}

/// Translates an error received from the FFI, into an [`anyhow::Error`].
unsafe fn translate_ffi_error(result: ffi::LevelResult) -> anyhow::Error {
    debug_assert_ne!(result.status, LoadStatus::Success, "Attempt to translate a success status into an error");

    // SAFETY: This string is guaranteed to have a null termination character,
    // as it has been created by the c_str method on std::string in C++.
    let ffi_err = CStr::from_ptr(result.data as *const c_char);
    let str = ffi_err.to_string_lossy();

    // Deallocate original string, now that it is converted into an owned Rust string.
    // SAFETY: The data has not been modified and has been allocated by C++.
    // It is therefore safe to deallocate.
    ffi::level_deallocate_array(result.data as *mut c_char);

    anyhow!(str)
}
