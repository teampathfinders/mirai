#ifdef __cplusplus
extern "C" {
#endif __cplusplus

    // Result returned by fallible operations.
    // Data must manually be freed after use.
    struct LevelResult {
        int is_success;
        int size;
        void* data;
    };

    // Open a LevelDB database.
    struct LevelResult level_open_database(const char* path);
    // Close a LevelDB database.
    // This also frees the pointers, it must no longer be used.
    void level_close_database(void* database);
    // Loads a key from the database.
    struct LevelResult level_get_key(void* database, const char* key, int key_size);
    // Deallocates a string previously allocated by another function.
    void level_deallocate_array(char* array);
    // Creates an iterator that iterates of all the keys.
    struct LevelResult level_iter(void* database);
    // Destroys an iterator previously created with level_iter.
    void level_destroy_iter(void* iter);
    // Returns the current key from the iterator.
    // SAFETY: The caller must ensure the iterator is still valid before calling this.
    LevelResult level_iter_key(const void* iter);
    // SAFETY: The caller must ensure the iterator is still valid before calling this.
    LevelResult level_iter_value(const void* iter);
    // Returns whether the iterator is still valid.
    bool level_iter_valid(const void* iter);
    // Moves the iterator to the next position.
    // This position could be invalid.
    void level_iter_next(void* iter);
#ifdef __cplusplus
}
#endif __cplusplus