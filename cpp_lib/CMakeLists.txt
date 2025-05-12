cmake_minimum_required(VERSION 3.30)
project(cpp_lib)

set(CMAKE_CXX_STANDARD 14)

include_directories(./)

add_executable(cpp_lib main.cpp)

add_subdirectory(tests)
