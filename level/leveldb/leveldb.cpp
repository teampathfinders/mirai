#include "leveldb.h"

#include <memory>

#include <leveldb/cache.h>
#include <leveldb/db.h>
#include <leveldb/decompress_allocator.h>
#include <leveldb/env.h>
#include <leveldb/filter_policy.h>
#include <leveldb/options.h>
#include <leveldb/status.h>
#include <leveldb/zlib_compressor.h>

enum DbStatus translate_status(const leveldb::Status& status) noexcept {
  return static_cast<DbStatus>(status.code());
}

class NoOpLogger : public leveldb::Logger {
public:
  void Logv(const char *fmt, va_list args) override {}
};

struct Database {
  leveldb::Options options = leveldb::Options();
  leveldb::WriteOptions write_options{};
  leveldb::ReadOptions read_options{};
  leveldb::DB *database = nullptr;

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

LevelResult level_open(const char *path) {
  LevelResult result{};

  std::unique_ptr<Database> database = std::make_unique<Database>();

  database->options.filter_policy = leveldb::NewBloomFilterPolicy(10);
  database->options.block_cache = leveldb::NewLRUCache(40 * 1024 * 1024);
  database->options.info_log = new NoOpLogger();
  database->options.compressors[0] = new leveldb::ZlibCompressorRaw();
  database->options.compressors[1] = new leveldb::ZlibCompressor();
  database->read_options.decompress_allocator =
      new leveldb::DecompressAllocator();

  leveldb::Status status =
      leveldb::DB::Open(database->options, path, &database->database);

  result.status = translate_status(status);
  if (status.ok()) {
    result.size = sizeof(Database);
    result.data = database.release();
  } else {
    std::string cpp_src = status.ToString();
    const char *src = cpp_src.c_str();
    size_t src_size = cpp_src.size() + 1; // Make space for null terminator.

    result.size = static_cast<int>(src_size);
    result.data = new char[src_size];
    memcpy(result.data, src, src_size);
  }

  return result;
}

void level_close(void *database_ptr) {
  auto database = reinterpret_cast<Database *>(database_ptr);
  delete database;
}

LevelResult level_get(void *database_ptr, const char *key, int key_size) {
  LevelResult result{};

  auto database = reinterpret_cast<Database *>(database_ptr);
  std::string value;

  auto status = database->database->Get(database->read_options,
                                        leveldb::Slice(key, key_size), &value);

  result.status = translate_status(status);
  if (status.ok()) {
    result.size = static_cast<int>(value.size());
    result.data = new char[value.size()];

    memcpy(result.data, value.data(), value.size());
  } else {
    std::string error = status.ToString();
    const char *src = error.c_str();
    size_t src_size = error.size() + 1; // Make space for null terminator.

    result.size = static_cast<int>(src_size);
    result.data = new char[src_size];
    memcpy(result.data, src, src_size);
  }
  
  return result;
}

LevelResult level_insert(void *database_ptr, const char *key, int key_size,
                         const char *value, int value_size) {
  auto database = reinterpret_cast<Database *>(database_ptr);
  LevelResult result{};

  leveldb::Slice key_slice(key, key_size);
  leveldb::Slice value_slice(value, value_size);

  auto status =
      database->database->Put(database->write_options, key_slice, value_slice);

  result.status = translate_status(status);
  if (status.ok()) {
    result.data = nullptr;
    result.size = 0;
  } else {
    auto error = status.ToString();
    auto src = error.c_str();
    size_t src_size = error.size() + 1; // Make space for null terminator.

    result.size = src_size;
    result.data = new char[src_size];
    memcpy(result.data, src, src_size);
  }

  return result;
}

LevelResult level_remove(void *database_ptr, const char *key, int key_size) {
  auto database = reinterpret_cast<Database *>(database_ptr);
  LevelResult result{};

  leveldb::Slice key_slice(key, key_size);

  auto status = database->database->Delete(database->write_options, key_slice);

  result.status = translate_status(status);
  if (status.ok()) {
    result.data = nullptr;
    result.size = 0;
  } else {
    auto error = status.ToString();
    auto src = error.c_str();
    size_t src_size = error.size() + 1; // Make space for null terminator.

    result.size = src_size;
    result.data = new char[src_size];
    memcpy(result.data, src, src_size);
  }

  return result;
}

void level_deallocate_array(char *array) { delete[] array; }

LevelResult level_iter(void *database) {
  auto db = reinterpret_cast<Database *>(database);

  leveldb::Iterator *iter = db->database->NewIterator(db->read_options);
  iter->SeekToFirst();

  LevelResult result{};
  result.status = DbStatus::Success;
  result.size = static_cast<int>(sizeof(leveldb::Iterator));
  result.data = iter;

  return result;
}

void level_destroy_iter(void *iter_raw) {
  auto iter = reinterpret_cast<leveldb::Iterator *>(iter_raw);
  delete iter;
}

LevelResult level_iter_key(const void *iter_raw) {
  auto iter = reinterpret_cast<const leveldb::Iterator *>(iter_raw);
  leveldb::Slice key = iter->key();

  LevelResult result{};
  result.status = DbStatus::Success;
  result.size = key.size();
  result.data = new char[result.size];
  memcpy(result.data, key.data(), result.size);

  return result;
}

LevelResult level_iter_value(const void *iter_raw) {
  auto iter = reinterpret_cast<const leveldb::Iterator *>(iter_raw);
  leveldb::Slice value = iter->value();

  LevelResult result{};
  result.status = DbStatus::Success;
  result.size = value.size();
  result.data = new char[result.size];
  memcpy(result.data, value.data(), result.size);

  return result;
}

void level_iter_next(void *iter_raw) {
  auto iter = reinterpret_cast<leveldb::Iterator *>(iter_raw);
  iter->Next();
}

bool level_iter_valid(const void *iter_raw) {
  auto iter = reinterpret_cast<const leveldb::Iterator *>(iter_raw);
  return iter->Valid();
}