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

#ifdef __cplusplus
}
#endif __cplusplus