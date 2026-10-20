cmake_minimum_required(VERSION 3.16)
project(cpp_lib CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# -DBITIS_SANITIZE=ON builds the tests with AddressSanitizer + UndefinedBehaviorSanitizer
option(BITIS_SANITIZE "Build tests with ASan and UBSan" OFF)

include_directories(./)

add_executable(cpp_lib main.cpp)

enable_testing()
add_subdirectory(tests)
