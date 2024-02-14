#pragma once

#ifndef LEVELDB_DECOMPRESS_ALLOCATOR_H_
#define LEVELDB_DECOMPRESS_ALLOCATOR_H_

#include <mutex>
#include <vector>
#include <string>

// Suppress dll-interface warning
#if _MSC_VER && !__INTEL_COMPILER
#pragma warning(push)
#pragma warning(disable : 4251)
#endif

namespace leveldb {
    class DLLX DecompressAllocator {
        public:
                virtual ~DecompressAllocator();

                virtual std::string get();
                virtual void release(std::string&& string);

                virtual void prune();

        protected:
                std::mutex mutex;
                std::vector<std::string> stack;
    };
}

#if _MSC_VER && !__INTEL_COMPILER
#pragma warning(pop)
#endif

#endif
