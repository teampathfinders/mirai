#ifdef __cplusplus
extern "C" {
#endif __cplusplus

enum DbStatus {
    Success,
    NotFound,
    Corruption,
    NotSupported,
    InvalidArgument,
    IOError
};

// Result returned by fallible operations.
// Data must manually be freed after use.
struct LevelResult {
    enum DbStatus status;
    int size;
    void *data;
};

struct SizedData {
    int size;
    void* data;
};

// Open a LevelDB database.
struct LevelResult level_open(const char *path);

// Close a LevelDB database.
// This also frees the pointers, it must no longer be used.
void level_close(void *database);

// Loads a key from the database.
struct LevelResult level_get(void *database, const char *key, int key_size);

/// Writes a value into the database.
struct LevelResult level_insert(void *database_ptr, const char *key,
                                int key_size, const char *value,
                                int value_size);

/// Removes a key from the database.
struct LevelResult level_remove(void *database_ptr, const char *key, int key_size);

// Deallocates a string previously allocated by another function.
void level_deallocate_array(char *array);

// Creates an iterator that iterates of all the keys.
struct SizedData level_iter(void *database);

// Destroys an iterator previously created with level_iter.
void level_destroy_iter(void *iter);

// Returns the current key from the iterator.
// SAFETY: The caller must ensure the iterator is still valid before calling
// this.
struct SizedData level_iter_key(const void *iter);

// SAFETY: The caller must ensure the iterator is still valid before calling
// this.
struct SizedData level_iter_value(const void *iter);

// Returns whether the iterator is still valid.
bool level_iter_valid(const void *iter);

// Moves the iterator to the next position.
// This position could be invalid.
void level_iter_next(void *iter);

#ifdef __cplusplus
}
#endif __cplusplus