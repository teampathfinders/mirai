use std::{
    ffi::{c_int, c_void},
    os::raw::c_char,
};

#[derive(Debug)]
#[repr(C)]
pub struct LevelResult {
    pub is_success: c_int,
    pub size: c_int,
    pub data: *mut c_void,
}

extern "C" {
    /// Open a LevelDB database.
    pub fn level_open_database(path: *const c_char) -> LevelResult;
    /// Close a LevelDB database.
    /// This also frees the pointers, it must no longer be used.
    pub fn level_close_database(database: *mut c_void);
    /// Loads a key from the database.
    pub fn level_get_key(
        database: *mut c_void, key: *const c_char, key_size: c_int,
    ) -> LevelResult;
    /// Deallocates a string previously allocated by another function.
    pub fn level_deallocate_array(array: *mut c_char);
}
