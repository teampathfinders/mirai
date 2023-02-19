use std::{
    ffi::{c_void, CStr, CString},
    os::raw::{c_char, c_int},
};

use bytes::{Bytes, BytesMut};
use common::{error, VError, VResult};

use crate::ffi;

/// Rust interface around a C++ LevelDB database.
#[derive(Debug)]
pub struct ChunkDatabase {
    /// Pointer to the C++ Database struct, containing the database and corresponding options.
    /// This data is heap-allocated and must therefore also be deallocated by C++ when it is no longer needed.
    pointer: *mut c_void,
}

impl ChunkDatabase {
    /// Opens the database at the specified path.
    pub fn new<P: AsRef<str>>(path: P) -> VResult<Self> {
        Ok(Self {
            pointer: std::ptr::null_mut()
        })

        // let ffi_path = CString::new(path.as_ref())?;
        // let result = unsafe {
        //     // SAFETY: This function is guaranteed to not return exceptions.
        //     // It also does not modify the argument and returns a valid struct.
        //     ffi::level_open_database(ffi_path.as_ptr())
        // };

        // if result.is_success == 1 {
        //     Ok(Self { pointer: result.data })
        // } else {
        //     Err(translate_ffi_error(result))
        // }
    }

    /// Loads the value of the given key.
    /// This function requires a raw key, i.e. the key must have been serialised already.
    pub fn get_raw_key<K: AsRef<[u8]>>(&self, key: K) -> VResult<Bytes> {
        let key = key.as_ref();
        let result = unsafe {
            // SAFETY: This function is guaranteed to not modify any arguments.
            // It also does not throw exceptions and returns a valid struct.
            //
            // LevelDB is thread-safe, this function can be used on multiple threads.
            ffi::level_get_key(
                self.pointer,
                key.as_ptr() as *mut c_char,
                key.len() as c_int,
            )
        };

        if result.is_success == 1 {
            let data = unsafe {
                std::slice::from_raw_parts(
                    result.data as *mut u8,
                    result.size as usize,
                )
            };

            let buffer = Bytes::from(data);

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

impl Drop for ChunkDatabase {
    fn drop(&mut self) {
        // Make sure to clean up the LevelDB resources when the database is dropped.
        // This can only be done by C++.
        unsafe {
            ffi::level_close_database(self.pointer);
        }
    }
}

/// SAFETY: The LevelDB authors explicitly state their database is thread-safe.
unsafe impl Send for ChunkDatabase {}
/// SAFETY: The LevelDB authors explicitly state their database is thread-safe.
unsafe impl Sync for ChunkDatabase {}

/// Translates an error received from the FFI, into a [`VError`].
fn translate_ffi_error(result: ffi::LevelResult) -> VError {
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
