include(GoogleTest)

# googletest is fetched from GitHub; for offline builds pass
#   -DFETCHCONTENT_SOURCE_DIR_GOOGLETEST=<path to a googletest checkout>
include(FetchContent)
FetchContent_Declare(
        googletest
        URL https://github.com/google/googletest/archive/03597a01ee50ed33e9dfd640b249b4be3799d395.zip
)
# For Windows: Prevent overriding the parent project's compiler/linker settings
set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(googletest)

enable_testing()

add_executable(
        bitis_testing

        test.cpp
        test_safety.cpp
        test_oneof.cpp
        test_xl.cpp
        test_fuzz.cpp
        test_bounds.cpp
        test_footprint.cpp
       )
target_include_directories(bitis_testing PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})
target_link_libraries(bitis_testing GTest::gtest_main)

# the header (and code generated with it) has to compile warning free, without
# exceptions and RTTI (ESP32 firmware settings)
add_executable(bitis_header_check header_check.cpp)
target_include_directories(bitis_header_check PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})
if (CMAKE_CXX_COMPILER_ID MATCHES "GNU|Clang")
    target_compile_options(bitis_header_check PRIVATE -Wall -Wextra -Wpedantic -Werror -fno-exceptions -fno-rtti)
    target_compile_options(bitis_testing PRIVATE -Wall -Wextra -Wno-missing-field-initializers)
endif()
add_test(NAME bitis_header_check COMMAND bitis_header_check)

# BITIS_NO_PRINT: print() is empty and printf is not referenced (separate executable,
# print() differs from the other test files)
add_library(bitis_no_print_obj OBJECT no_print_check.cpp)
target_include_directories(bitis_no_print_obj PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})
add_executable(bitis_no_print_check $<TARGET_OBJECTS:bitis_no_print_obj>)
if (CMAKE_CXX_COMPILER_ID MATCHES "GNU|Clang")
    target_compile_options(bitis_no_print_obj PRIVATE -Wall -Wextra -Wpedantic -Werror -fno-exceptions -fno-rtti)
endif()
add_test(NAME bitis_no_print_check COMMAND bitis_no_print_check)
add_test(NAME bitis_no_print_symbols COMMAND ${CMAKE_COMMAND} -DNM=${CMAKE_NM}
        "-DOBJECTS=$<TARGET_OBJECTS:bitis_no_print_obj>" -P ${CMAKE_CURRENT_SOURCE_DIR}/check_no_print_symbols.cmake)

# golden vectors shared with the Rust and Dart runtimes (golden/README.md)
set(BITIS_GOLDEN_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../../golden)
add_executable(bitis_golden ${BITIS_GOLDEN_DIR}/cpp/golden_test.cpp)
target_include_directories(bitis_golden PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/.. ${BITIS_GOLDEN_DIR}/gen/cpp)
target_compile_definitions(bitis_golden PRIVATE BITIS_GOLDEN_MANIFEST="${BITIS_GOLDEN_DIR}/vectors/manifest.txt")
target_link_libraries(bitis_golden GTest::gtest_main)
if (CMAKE_CXX_COMPILER_ID MATCHES "GNU|Clang")
    target_compile_options(bitis_golden PRIVATE -Wall -Wextra -Wno-missing-field-initializers)
endif()

if (BITIS_SANITIZE)
    foreach(tgt bitis_testing bitis_header_check bitis_golden bitis_no_print_obj bitis_no_print_check)
        target_compile_options(${tgt} PRIVATE -fsanitize=address,undefined -fno-sanitize-recover=undefined -fno-omit-frame-pointer -g)
        target_link_options(${tgt} PRIVATE -fsanitize=address,undefined)
    endforeach()
endif()

gtest_discover_tests(bitis_testing)
gtest_discover_tests(bitis_golden)

# CRC-32 helper bitis_crc32.h: gtests against golden/vectors/crc32.txt (nibble table and
# BITIS_CRC32_TABLE256) are part of bitis_golden, so golden/run_all.sh runs them (also with
# ASan/UBSan); the strict header checks are built together with bitis_golden.
target_sources(bitis_golden PRIVATE test_crc32.cpp test_crc32_table256.cpp)
target_compile_definitions(bitis_golden PRIVATE BITIS_GOLDEN_CRC32="${BITIS_GOLDEN_DIR}/vectors/crc32.txt")
foreach(variant nibble16 table256)
    add_executable(bitis_crc32_check_${variant} crc32_header_check.cpp)
    target_include_directories(bitis_crc32_check_${variant} PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/..)
    if (variant STREQUAL "table256")
        target_compile_definitions(bitis_crc32_check_${variant} PRIVATE BITIS_CRC32_TABLE256)
    endif()
    if (CMAKE_CXX_COMPILER_ID MATCHES "GNU|Clang")
        target_compile_options(bitis_crc32_check_${variant} PRIVATE -Wall -Wextra -Wpedantic -Werror -fno-exceptions -fno-rtti)
    endif()
    add_test(NAME bitis_crc32_check_${variant} COMMAND bitis_crc32_check_${variant})
    add_dependencies(bitis_golden bitis_crc32_check_${variant})
endforeach()
