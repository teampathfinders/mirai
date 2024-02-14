#ifndef PATHFINDERS_LEVELDB_SUPPORT_H
#define PATHFINDERS_LEVELDB_SUPPORT_H

#include <cstdint>

using ssize_t = intptr_t;

// Static libraries don't need anything
#define DLLX

// Used for shared library
//#ifdef LEVELDB_EXPORT
//    #ifdef WIN32
//        #define DLLX __declspec(dllexport)
//    #else
//        #define DLLX
//    #endif
//#else
//    #ifdef WIN32
//        #define DLLX __declspec(dllimport)
//    #else
//        #define DLLX
//    #endif
//#endif

#endif // PATHFINDERS_LEVELDB_SUPPORT_H
