cmake_minimum_required(VERSION 3.5.0)
project(ohos_app_bridge)

set(NATIVERENDER_ROOT_PATH ${CMAKE_CURRENT_SOURCE_DIR})

add_library(rust_bridge STATIC IMPORTED)
set_target_properties(rust_bridge PROPERTIES
    IMPORTED_LOCATION ${NATIVERENDER_ROOT_PATH}/libs/arm64-v8a/libohos_app.a
)

add_library(entry SHARED napi_init.cpp)
set_target_properties(entry PROPERTIES
    LIBRARY_OUTPUT_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/out
)
target_link_libraries(entry PUBLIC libace_napi.z.so rust_bridge)
