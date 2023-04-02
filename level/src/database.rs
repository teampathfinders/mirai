use std::{
    ffi::{c_void, CStr, CString},
    marker::PhantomData,
    ops::Deref,
    os::raw::{c_char, c_int},
};
use std::ptr::NonNull;
use anyhow::anyhow;

use util::{error, Error, Result};

use crate::ffi;

/// Wraps a LevelDB buffer, ensuring the buffer is deallocated after use.
#[derive(Debug)]
pub struct Guard<'a>(&'a [u8]);

impl<'a> Guard<'a> {
    // This is not implemented as a `From` trait so that `Guard` can only be constructed
    // inside of this crate.
    // SAFETY: A `Guard` must only be created from a slice that was allocated by
    // `LevelDb` or `Keys`.
    // The caller must also ensure that the slice is not referenced anywhere else in the program.
    #[inline]
    pub(crate) unsafe fn from_slice(slice: &'a [u8]) -> Self {
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
            ffi::level_deallocate_array(self.0.as_ptr() as *mut i8);
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
    pub fn key(&self) -> Guard {
        // SAFETY: A Ref should only exist while the iterator is valid.
        // This invariant is upheld by the lifetime 'a.
        unsafe {
            let result = ffi::level_iter_key(self.iter.as_ptr());
            debug_assert_eq!(result.is_success, 1);

            Guard::from_slice(std::slice::from_raw_parts(
                result.data as *const u8,
                result.size as usize,
            ))
        }
    }

    /// The data associated with this pair.
    pub fn value(&self) -> Guard {
        // SAFETY: A Ref should only exist while the iterator is valid.
        // This invariant is upheld by the lifetime 'a.
        unsafe {
            let result = ffi::level_iter_value(self.iter.as_ptr());
            debug_assert_eq!(result.is_success, 1);

            Guard::from_slice(std::slice::from_raw_parts(
                result.data as *const u8,
                result.size as usize,
            ))
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
    pub fn new(db: &'a Database) -> Self {
        // SAFETY: level_iter is guaranteed to not return an error.
        // The iterator position has also been initialized by FFI and is not in an invalid state.
        let result = unsafe { ffi::level_iter(db.ptr.as_ptr()) };

        debug_assert_eq!(result.is_success, 1);
        debug_assert_ne!(result.data, std::ptr::null_mut());

        Self {
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
        if valid {
            let raw_ref = KvRef { iter: self.iter, _marker: PhantomData };
            Some(raw_ref)
        } else {
            None
        }
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
    pub fn open<P>(path: P) -> anyhow::Result<Self>
        where
            P: AsRef<str>,
    {
        let ffi_path = CString::new(path.as_ref())?;

        unsafe {
            // SAFETY: This function is guaranteed to not return exceptions.
            // It also does not modify the argument and returns a valid struct.
            let result = ffi::level_open(ffi_path.as_ptr());

            if result.is_success == 1 {
                debug_assert_ne!(result.data, std::ptr::null_mut());

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
    pub fn get<K>(&self, key: K) -> anyhow::Result<Guard>
        where
            K: AsRef<[u8]>,
    {
        let key = key.as_ref();
        unsafe {
            // SAFETY: This function is guaranteed to not modify any arguments.
            // It also does not throw exceptions and returns a valid struct.
            //
            // LevelDB is thread-safe, this function can be used by multiple threads.
            let result = ffi::level_get(
                self.ptr.as_ptr(),
                key.as_ptr() as *const c_char,
                key.len() as c_int,
            );

            if result.is_success == 1 {
                debug_assert_ne!(result.data, std::ptr::null_mut());

                // SAFETY: result.data is guaranteed by the caller to be a valid pointer.
                // result.size is also guaranteed to be the size of the actual array.
                let data = std::slice::from_raw_parts(
                    result.data as *const u8,
                    result.size as usize,
                );

                // SAFETY: The data passed into the Guard has been allocated in the leveldb FFI code.
                // It is therefore also required to deallocate the data there, which is what Guard
                // does.
                Ok(Guard::from_slice(data))
            } else {
                Err(translate_ffi_error(result))
            }
        }
    }

    pub fn insert<K, V>(&self, key: K, value: V) -> anyhow::Result<()>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>
    {
        let key = key.as_ref();
        let value = value.as_ref();

        unsafe {
            let result = ffi::level_insert(
                self.ptr.as_ptr(),
                key.as_ptr() as *const c_char,
                key.len() as c_int,
                value.as_ptr() as *const c_char,
                value.len() as c_int
            );

            if result.is_success == 1 {
                Ok(())
            } else {
                Err(translate_ffi_error(result))
            }
        }
    }

    pub fn remove<K>(&self, key: K) -> anyhow::Result<()>
    where
        K: AsRef<[u8]>
    {
        let key = key.as_ref();

        unsafe {
            let result = ffi::level_remove(
                self.ptr.as_ptr(),
                key.as_ptr() as *mut c_char,
                key.len() as c_int
            );

            if result.is_success == 1 {
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
        // Make sure to clean up the LevelDB resources when the database is dropped.
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

/// Translates an error received from the FFI, into an [`Error`].
unsafe fn translate_ffi_error(result: ffi::LevelResult) -> anyhow::Error {
    debug_assert_eq!(result.is_success, 0);

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
