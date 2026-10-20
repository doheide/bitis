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

if (BITIS_SANITIZE)
    foreach(tgt bitis_testing bitis_header_check)
        target_compile_options(${tgt} PRIVATE -fsanitize=address,undefined -fno-sanitize-recover=undefined -fno-omit-frame-pointer -g)
        target_link_options(${tgt} PRIVATE -fsanitize=address,undefined)
    endforeach()
endif()

gtest_discover_tests(bitis_testing)
