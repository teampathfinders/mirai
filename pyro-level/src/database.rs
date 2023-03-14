use std::{
    ffi::{c_void, CStr, CString},
    marker::PhantomData,
    ops::Deref,
    os::raw::{c_char, c_int},
};
use std::ptr::NonNull;

use bytes::{Bytes, BytesMut};
use util::{error, Error, Result};

use crate::ffi;

pub struct BufferGuard<'a> {
    buf: &'a [u8],
}

impl<'a> From<&'a [u8]> for BufferGuard<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self { buf: value }
    }
}

impl<'a> Deref for BufferGuard<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.buf
    }
}

impl<'a> Drop for BufferGuard<'a> {
    fn drop(&mut self) {
        unsafe {
            ffi::level_deallocate_array(self.buf.as_ptr() as *mut i8);
        }
    }
}

pub struct RawRef<'a> {
    iter: NonNull<c_void>,
    /// The 'a lifetime isn't used in the struct.
    /// It is only used to make sure that RawRef does not outlive RawDatabase and RawIter.
    phantom: PhantomData<&'a ()>,
}

impl<'a> RawRef<'a> {
    pub fn key(&self) -> BufferGuard<'a> {
        // SAFETY: A RawRef should only exist while the iterator is valid.
        unsafe {
            let result = ffi::level_iter_key(self.iter.as_ptr());
            debug_assert_eq!(result.is_success, 1);

            std::slice::from_raw_parts(
                result.data as *const u8,
                result.size as usize,
            )
            .into()
        }
    }

    pub fn value(&self) -> BufferGuard<'a> {
        // SAFETY: A RawRef should only exist while the iterator is valid.
        unsafe {
            let result = ffi::level_iter_value(self.iter.as_ptr());
            debug_assert_eq!(result.is_success, 1);

            std::slice::from_raw_parts(
                result.data as *const u8,
                result.size as usize,
            )
            .into()
        }
    }
}

pub struct RawKeyIter<'a> {
    index: usize,
    db: &'a RawDatabase,
    iter: NonNull<c_void>,
}

impl<'a> RawKeyIter<'a> {
    fn new(db: &'a RawDatabase) -> Self {
        // SAFETY: level_iter is guaranteed to not return an error.
        // The iterator position has also been initialized by FFI and is not in an invalid state.
        let result = unsafe { ffi::level_iter(db.pointer.as_ptr()) };

        debug_assert_eq!(result.is_success, 1);
        debug_assert_ne!(result.data, std::ptr::null_mut());

        Self {
            index: 0,
            db,
            // SAFETY: level_iter is guaranteed to not return an error.
            iter: unsafe {
                NonNull::new_unchecked(result.data)
            }
        }
    }
}

impl<'a> Iterator for RawKeyIter<'a> {
    type Item = RawRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index != 0 {
            unsafe { ffi::level_iter_next(self.iter.as_ptr()) };
        }
        self.index += 1;

        let valid = unsafe { ffi::level_iter_valid(self.iter.as_ptr()) };

        if valid {
            let raw_ref = RawRef { iter: self.iter, phantom: PhantomData };

            Some(raw_ref)
        } else {
            None
        }
    }
}

impl<'a> Drop for RawKeyIter<'a> {
    fn drop(&mut self) {
        unsafe {
            ffi::level_destroy_iter(self.iter.as_ptr());
        }
    }
}

/// Rust interface around a C++ LevelDB database.
#[derive(Debug)]
pub struct RawDatabase {
    /// Pointer to the C++ Database struct, containing the database and corresponding options.
    /// This data is heap-allocated and must therefore also be deallocated by C++ when it is no longer needed.
    pointer: NonNull<c_void>
}

impl RawDatabase {
    /// Opens the database at the specified path.
    pub fn new<P: AsRef<str>>(path: P) -> Result<Self> {
        let ffi_path = CString::new(path.as_ref())?;
        let result = unsafe {
            // SAFETY: This function is guaranteed to not return exceptions.
            // It also does not modify the argument and returns a valid struct.
            ffi::level_open_database(ffi_path.as_ptr())
        };

        if result.is_success == 1 {
            debug_assert_ne!(result.data, std::ptr::null_mut());
            // SAFETY: If result.is_success is true, then the caller has set data to a valid pointer.
            Ok(Self { pointer: unsafe {
                NonNull::new_unchecked(result.data)
            }})
        } else {
            Err(translate_ffi_error(result))
        }
    }

    pub fn iter(&self) -> RawKeyIter {
        RawKeyIter::new(self)
    }

    /// Loads the value of the given key.
    /// This function requires a raw key, i.e. the key must have been serialised already.
    pub fn get_raw_key<K: AsRef<[u8]>>(&self, key: K) -> Result<Bytes> {
        let key = key.as_ref();
        let result = unsafe {
            // SAFETY: This function is guaranteed to not modify any arguments.
            // It also does not throw exceptions and returns a valid struct.
            //
            // LevelDB is thread-safe, this function can be used on multiple threads.
            ffi::level_get_key(
                self.pointer.as_ptr(),
                key.as_ptr() as *mut c_char,
                key.len() as c_int,
            )
        };

        if result.is_success == 1 {
            dbg!(result.size);

            let data = unsafe {
                std::slice::from_raw_parts(
                    result.data as *mut u8,
                    result.size as usize,
                )
            };
            // println!("data = {:?}", &data[..15]);

            let buffer = Bytes::copy_from_slice(data);
            unsafe {
                // SAFETY: Data is safe to deallocate because BytesMut copies the data.
                // and it is not used anywhere else.
                ffi::level_deallocate_array(result.data as *mut c_char)
            };

            Ok(buffer)
        } else {
            Err(translate_ffi_error(result))
        }
    }
}

impl Drop for RawDatabase {
    fn drop(&mut self) {
        // Make sure to clean up the LevelDB resources when the database is dropped.
        // This can only be done by C++.
        unsafe {
            ffi::level_close_database(self.pointer.as_ptr());
        }
    }
}

/// SAFETY: The LevelDB authors explicitly state their database is thread-safe.
unsafe impl Send for RawDatabase {}
/// SAFETY: The LevelDB authors explicitly state their database is thread-safe.
unsafe impl Sync for RawDatabase {}

/// Translates an error received from the FFI, into a [`Error`].
fn translate_ffi_error(result: ffi::LevelResult) -> Error {
    let ffi_err = unsafe {
        // SAFETY: This string is guaranteed to have a null termination character.
        CStr::from_ptr(result.data as *const c_char)
    };

    let str = ffi_err.to_string_lossy();

    // Deallocate original string, now that it is converted into an owned Rust string.
    unsafe {
        // SAFETY: The string is guaranteed to exist, it has not been modified.
        ffi::level_deallocate_array(result.data as *mut c_char)
    };

    error!(DatabaseFailure, str.to_string())
}
