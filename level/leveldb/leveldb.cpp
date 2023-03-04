#include "leveldb.h"

#include <memory>

#include <leveldb/options.h>
#include <leveldb/filter_policy.h>
#include <leveldb/cache.h>
#include <leveldb/env.h>
#include <leveldb/zlib_compressor.h>
#include <leveldb/decompress_allocator.h>
#include <leveldb/status.h>
#include <leveldb/db.h>

class EmptyLogger : public leveldb::Logger {
public:
    void Logv(const char* fmt, va_list args) override {}
};

struct Database {
    leveldb::Options options = leveldb::Options();
    leveldb::ReadOptions read_options = leveldb::ReadOptions();
    leveldb::DB* database = nullptr;

    ~Database() noexcept {
        delete this->database;

        delete this->read_options.decompress_allocator;

        delete this->options.compressors[1];
        delete this->options.compressors[0];
        delete this->options.info_log;
        delete this->options.block_cache;
        delete this->options.filter_policy;
    }
};

LevelResult level_open_database(const char* path) {
    LevelResult result{};

    try {
        std::unique_ptr<Database> database = std::make_unique<Database>();

        database->options.filter_policy = leveldb::NewBloomFilterPolicy(10);
        database->options.block_cache = leveldb::NewLRUCache(40 * 1024 * 1024);
        database->options.info_log = new EmptyLogger();
        database->options.compressors[0] = new leveldb::ZlibCompressorRaw();
        database->options.compressors[1] = new leveldb::ZlibCompressor();

        database->read_options.decompress_allocator = new leveldb::DecompressAllocator();

        leveldb::Status status = leveldb::DB::Open(database->options, path, &database->database);

        result.is_success = status.ok();

        if(!status.ok()) {
            std::string cpp_src = status.ToString();
            const char* src = cpp_src.c_str();
            size_t src_size = cpp_src.size() + 1; // Make space for null terminator.

            result.size = static_cast<int>(src_size);
            result.data = new char[src_size];
            memcpy(result.data, src, src_size);

            return result;
        }

        result.size = sizeof(Database);
        result.data = database.release();
    } catch(const std::exception& e) {
        result.is_success = false;

        const char* src = e.what();
        size_t src_size = strlen(src) + 1; // Make space for null terminator.

        result.size = static_cast<int>(src_size);
        result.data = new char[src_size];
        memcpy(result.data, src, src_size);
    } catch(...) {
        // No error message
        result.is_success = false;
        result.size = 0;
        result.data = nullptr;
    }

    return result;
}

void level_close_database(void* database_ptr) {
    auto database = reinterpret_cast<Database*>(database_ptr);
    delete database;
}

LevelResult level_get_key(void* database_ptr, const char* key, int key_size) {
    LevelResult result{};

    try {
        auto database = reinterpret_cast<Database*>(database_ptr);
        std::string value;

        auto status = database->database->Get(database->read_options, leveldb::Slice(key, key_size), &value);
        if(!status.ok()) {
            std::string cpp_src = status.ToString();
            const char* src = cpp_src.c_str();
            size_t src_size = cpp_src.size() + 1; // Make space for null terminator.

            result.size = static_cast<int>(src_size);
            result.data = new char[src_size];
            memcpy(result.data, src, src_size);

            return result;
        }

        result.is_success = true;
        result.size = static_cast<int>(value.size());
        result.data = new char[value.size()];

        memcpy(result.data, value.data(), value.size());
    } catch(const std::exception& e) {
        result.is_success = false;

        const char* src = e.what();
        size_t src_size = strlen(src) + 1; // Make space for null terminator.

        result.size = static_cast<int>(src_size);
        result.data = new char[src_size];
        memcpy(result.data, src, src_size);
    } catch(...) {
        result.is_success = false;
        result.size = 0;
        result.data = nullptr;
    }

    return result;
}

void level_deallocate_array(char* array) {
    delete[] array;
}

LevelResult level_iter(void* database) {
    auto db = reinterpret_cast<Database*>(database);

    leveldb::Iterator* iter = db->database->NewIterator(db->read_options);
    iter->SeekToFirst();

    LevelResult result{};
    result.is_success = true;
    result.size = static_cast<int>(sizeof(leveldb::Iterator));
    result.data = iter;

    return result;
}

void level_destroy_iter(void* iter_raw) {
    auto iter = reinterpret_cast<leveldb::Iterator*>(iter_raw);
    delete iter;
}

LevelResult level_iter_key(const void* iter_raw) {
    auto iter = reinterpret_cast<const leveldb::Iterator*>(iter_raw);
    leveldb::Slice key = iter->key();

    LevelResult result{};
    result.is_success = 1;
    result.size = key.size();
    result.data = new char[result.size];
    memcpy(result.data, key.data(), result.size);

    return result;
}

LevelResult level_iter_value(const void* iter_raw) {
    auto iter = reinterpret_cast<const leveldb::Iterator*>(iter_raw);
    leveldb::Slice value = iter->value();

    LevelResult result{};
    result.is_success = 1;
    result.size = value.size();
    result.data = new char[result.size];
    memcpy(result.data, value.data(), result.size);

    return result;
}

void level_iter_next(void* iter_raw) {
    auto iter = reinterpret_cast<leveldb::Iterator*>(iter_raw);
    iter->Next();
}

bool level_iter_valid(const void* iter_raw) {
    auto iter = reinterpret_cast<const leveldb::Iterator*>(iter_raw);
    return iter->Valid();
}