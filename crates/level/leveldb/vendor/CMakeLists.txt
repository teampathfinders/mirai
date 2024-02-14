cmake_minimum_required(VERSION 3.13 FATAL_ERROR)
project(leveldb-mcpe C CXX)

set(CMAKE_CXX_STANDARD 11)
set(Source_Files
    "src/db/builder.cc"
    "src/db/c.cc"
    "src/db/db_impl.cc"
    "src/db/db_iter.cc"
    "src/db/dbformat.cc"
    "src/db/dumpfile.cc"
    "src/db/filename.cc"
    "src/db/log_reader.cc"
    "src/db/log_writer.cc"
    "src/db/memtable.cc"
    "src/db/repair.cc"
    "src/db/snappy_compressor.cc"
    "src/db/table_cache.cc"
    "src/db/version_edit.cc"
    "src/db/version_set.cc"
    "src/db/write_batch.cc"
    "src/db/zlib_compressor.cc"
    "src/helpers/memenv/memenv.cc"
    "src/port/port_posix.cc"
    "src/port/port_posix_sse.cc"
    "src/port/port_win.cc"
    "src/table/block.cc"
    "src/table/block_builder.cc"
    "src/table/filter_block.cc"
    "src/table/format.cc"
    "src/table/iterator.cc"
    "src/table/merger.cc"
    "src/table/table.cc"
    "src/table/table_builder.cc"
    "src/table/two_level_iterator.cc"
    "src/util/arena.cc"
    "src/util/bloom.cc"
    "src/util/cache.cc"
    "src/util/coding.cc"
    "src/util/comparator.cc"
    "src/util/crc32c.cc"
    "src/util/env.cc"
    "src/util/env_posix.cc"
    "src/util/env_win.cc"
    "src/util/filter_policy.cc"
    "src/util/hash.cc"
    "src/util/histogram.cc"
    "src/util/logging.cc"
    "src/util/options.cc"
    "src/util/status.cc"
    "src/util/win_logger.cc"
    "lib/zlib/adler32.c"
    "lib/zlib/compress.c"
    "lib/zlib/crc32.c"
    "lib/zlib/deflate.c"
    "lib/zlib/gzclose.c"
    "lib/zlib/gzlib.c"
    "lib/zlib/gzread.c"
    "lib/zlib/gzwrite.c"
    "lib/zlib/infback.c"
    "lib/zlib/inffast.c"
    "lib/zlib/inflate.c"
    "lib/zlib/inftrees.c"
    "lib/zlib/trees.c"
    "lib/zlib/uncompr.c"
    "lib/zlib/zutil.c"
)

add_subdirectory("lib/zlib")
add_library(${PROJECT_NAME} STATIC ${Source_Files})
target_include_directories(${PROJECT_NAME} PUBLIC "lib" "include/port" "include")

# Supress warnings
if(MSVC)
    target_compile_options(${PROJECT_NAME} PRIVATE /w)
endif()

if(APPLE)
    target_compile_definitions(${PROJECT_NAME} PRIVATE -DOS_MACOSX)
endif()

if(UNIX)
    target_compile_definitions(${PROJECT_NAME} PRIVATE -DLEVELDB_PLATFORM_POSIX)
endif()
