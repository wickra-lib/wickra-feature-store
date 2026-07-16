# Build and run the C and C++ examples against the wickra-feature-store C ABI.
#
#   cargo build --release -p wickra-feature-store-c
#   cmake -S examples/c -B examples/c/build
#   cmake --build examples/c/build --config Release
#   ctest --test-dir examples/c/build -C Release --output-on-failure
#
# WICKRA_FEATURE_STORE_LIB_DIR defaults to the workspace target/release directory
# and can be overridden for out-of-tree builds.

cmake_minimum_required(VERSION 3.15)
project(wickra_feature_store_examples C CXX)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_CXX_STANDARD 14)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(WICKRA_FEATURE_STORE_LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../target/release"
    CACHE PATH "Directory containing the built wickra-feature-store C ABI library")

set(WICKRA_FEATURE_STORE_INCLUDE "${CMAKE_CURRENT_SOURCE_DIR}/../../bindings/c/include")

find_library(WICKRA_FEATURE_STORE_LIB
    NAMES wickra_feature_store libwickra_feature_store
    HINTS "${WICKRA_FEATURE_STORE_LIB_DIR}"
    REQUIRED)

add_executable(build_features build_features.c)
target_include_directories(build_features PRIVATE "${WICKRA_FEATURE_STORE_INCLUDE}")
target_link_libraries(build_features PRIVATE "${WICKRA_FEATURE_STORE_LIB}")

add_executable(build_featurespp build_features.cpp)
target_include_directories(build_featurespp PRIVATE "${WICKRA_FEATURE_STORE_INCLUDE}")
target_link_libraries(build_featurespp PRIVATE "${WICKRA_FEATURE_STORE_LIB}")

# On Windows the runtime DLL must sit next to the executable (there is no rpath);
# without it the loader pops a modal "DLL not found" dialog that hangs ctest. Copy
# it next to each example after build. (Unix resolves the .so/.dylib via the
# linker's embedded path, so this is Windows-only.)
if(WIN32)
    foreach(example build_features build_featurespp)
        add_custom_command(TARGET ${example} POST_BUILD
            COMMAND ${CMAKE_COMMAND} -E copy_if_different
                "${WICKRA_FEATURE_STORE_LIB_DIR}/wickra_feature_store.dll"
                "$<TARGET_FILE_DIR:${example}>"
            VERBATIM)
    endforeach()
endif()

enable_testing()
add_test(NAME build_features COMMAND build_features)
add_test(NAME build_featurespp COMMAND build_featurespp)
# A missing runtime dependency must fail fast, never block the runner for hours.
set_tests_properties(build_features build_featurespp PROPERTIES TIMEOUT 60)
