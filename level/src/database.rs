use std::ffi::{c_char, c_void, CStr, CString};

use common::{error, VError, VResult};

use crate::ffi;

pub struct Database {
    pointer: *mut c_void,
}

impl Database {
    pub fn new<P: AsRef<str>>(path: P) -> VResult<Self> {
        let ffi_path = CString::new(path.as_ref())?;
        let result = unsafe {
            // SAFETY: This function is guaranteed to not return exceptions.
            // It also does not modify the argument and returns a valid struct.
            ffi::level_open_database(ffi_path.as_ptr())
        };

        if result.is_success == 1 {
            Ok(Self {
                pointer: result.data,
            })
        } else {
            Err(translate_ffi_error(result))
        }
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        unsafe {
            ffi::level_close_database(self.pointer);
        }
    }
}

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
