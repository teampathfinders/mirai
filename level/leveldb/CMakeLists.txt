cmake_minimum_required(VERSION 3.0)
project(leveldb-wrapper)

add_subdirectory(vendor)
add_library(${PROJECT_NAME} STATIC leveldb.cpp)
target_include_directories(${PROJECT_NAME} PRIVATE vendor/include)
target_link_libraries(${PROJECT_NAME} PRIVATE leveldb-mcpe)