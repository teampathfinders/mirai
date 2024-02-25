#ifdef __cplusplus
extern "C" {
#endif __cplusplus

enum DbStatus {
    Success,
    NotFound,
    Corruption,
    NotSupported,
    InvalidArgument,
    IOError,
    AllocationFailed
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
struct LevelResult db_open(const char *path);

// Close a LevelDB database.
// This also frees the pointers, it must no longer be used.
void db_close(void *database);

// Loads a key from the database.
struct LevelResult db_get(void *database, const char *key, int key_size);

/// Writes a value into the database.
struct LevelResult db_put(void *database_ptr, const char *key,
                                int key_size, const char *value,
                                int value_size);

/// Removes a key from the database.
struct LevelResult db_delete(void *database_ptr, const char *key, int key_size);

// Deallocates a string previously allocated by another function.
void buffer_destroy(char *array);

// Creates an iterator that iterates of all the keys.
struct SizedData iter_new(void *database);

// Destroys an iterator previously created with level_iter.
void iter_destroy(void *iter);

// Returns the current key from the iterator.
// SAFETY: The caller must ensure the iterator is still valid before calling
// this.
struct SizedData iter_key(const void *iter);

// SAFETY: The caller must ensure the iterator is still valid before calling
// this.
struct SizedData iter_value(const void *iter);

// Returns whether the iterator is still valid.
bool iter_valid(const void *iter);

// Moves the iterator to the next position.
// This position could be invalid.
void iter_next(void *iter);

// Batched writes
// //////////////////////////////////////

/// Creates a new batch.
void* batch_new();

/// Adds a delete operation to the batch.
void batch_delete(void* batch, const char* key, int key_size);

/// Adds a put operation to the batch.
void batch_put(void* batch, const char* key, int key_size, const char* val, int val_size);

/// Clears all operations from the batch .
///
/// This can be used to reuse batches.
void batch_clear(void* batch);

// Completely destroys the batch.
// This should be called when the batch is no longer in use.
//
// The batch pointer should no longer be used after calling this.
void batch_destroy(void* batch);

/// Executes the batch on the specified database.
struct LevelResult batch_execute(void* db, void* batch);

#ifdef __cplusplus
}
#endif __cplusplus