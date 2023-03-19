cmake_minimum_required(VERSION 3.0)
project(leveldb-wrapper)

# Setting output directories in CMake is such a pain
# Why are there like 20 different ways to do it
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY_RELEASE ${CMAKE_BINARY_DIR}/out)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY_RELEASE ${CMAKE_BINARY_DIR}/out)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY_RELEASE ${CMAKE_BINARY_DIR}/out)

add_subdirectory(vendor)
add_library(${PROJECT_NAME} STATIC leveldb.cpp)
target_include_directories(${PROJECT_NAME} PRIVATE vendor/include)
target_link_libraries(${PROJECT_NAME} PRIVATE leveldb-mcpe)