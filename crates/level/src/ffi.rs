use std::ffi::{c_char, c_int, c_void};

/// Status of the LevelDB operation result.
#[allow(dead_code)] // These enum variants are used by C++ through FFI but not in Rust code.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum LoadStatus {
    Success,
    NotFound,
    Corruption,
    NotSupported,
    InvalidArgument,
    IOError,
}

#[derive(Debug)]
#[repr(C)]
pub struct SizedData {
    pub size: c_int,
    pub data: *mut c_void,
}

/// Result returned by fallible LevelDB operations.
#[derive(Debug)]
#[repr(C)]
pub struct LevelResult {
    /// The status of the result.
    pub status: LoadStatus,
    /// Size in bytes of the `data` field.
    pub size: c_int,
    /// Data such as an error or a response.
    pub data: *mut c_void,
}

extern "C" {
    /// Open a LevelDB database.
    pub fn level_open(path: *const c_char) -> LevelResult;
    /// Close a LevelDB database.
    /// This also frees the pointers, it must no longer be used.
    pub fn level_close(database: *mut c_void);
    /// Loads a value from the database.
    pub fn level_get(database: *mut c_void, key: *const c_char, key_size: c_int) -> LevelResult;
    /// Writes a value into the database.
    pub fn level_insert(database: *mut c_void, key: *const c_char, key_size: c_int, value: *const c_char, value_size: c_int) -> LevelResult;
    /// Deletes a key from the database.
    pub fn level_remove(database: *mut c_void, key: *const c_char, key_size: c_int) -> LevelResult;
    /// Deallocates a string previously allocated by another function.
    pub fn level_deallocate_array(array: *mut c_char);
    /// Creates an iterator over the database keys.
    pub fn level_iter(database: *mut c_void) -> SizedData;
    /// Destroys an iterator previously created with [`level_iter`].
    pub fn level_destroy_iter(iter: *mut c_void);
    /// Whether the iterator is still valid.
    pub fn level_iter_valid(iter: *const c_void) -> bool;
    /// The current key the iterator is on.
    pub fn level_iter_key(iter: *const c_void) -> SizedData;
    /// The current value the iterator is on.
    pub fn level_iter_value(iter: *const c_void) -> SizedData;
    /// Moves the iterator to the next position.
    pub fn level_iter_next(iter: *mut c_void);
}
