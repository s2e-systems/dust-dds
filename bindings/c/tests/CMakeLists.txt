cmake_minimum_required(VERSION 3.10)
project(dust_dds_c_test C)

set(CMAKE_C_STANDARD 99)

get_filename_component(BINDINGS_C_DIR "${CMAKE_CURRENT_SOURCE_DIR}/.." ABSOLUTE)
get_filename_component(WORKSPACE_ROOT "${BINDINGS_C_DIR}/../.." ABSOLUTE)

set(HEADER_DIR "${BINDINGS_C_DIR}/include")
set(LIB_DIR "${WORKSPACE_ROOT}/target/debug")

include_directories(${HEADER_DIR})
link_directories(${LIB_DIR})

add_executable(test_create_participant test_create_participant.c)

find_package(Threads REQUIRED)
target_link_libraries(test_create_participant PRIVATE dust_dds_c Threads::Threads ${CMAKE_DL_LIBS} m)
