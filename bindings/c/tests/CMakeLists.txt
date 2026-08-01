cmake_minimum_required(VERSION 3.14)
project(dust_dds_c_test C)

set(CMAKE_C_STANDARD 99)

# Enable CTest
enable_testing()

# Fetch Unity testing framework
include(FetchContent)
FetchContent_Declare(
    unity
    URL https://github.com/ThrowTheSwitch/Unity/archive/refs/tags/v2.6.0.tar.gz
)
FetchContent_MakeAvailable(unity)

get_filename_component(BINDINGS_C_DIR "${CMAKE_CURRENT_SOURCE_DIR}/.." ABSOLUTE)
get_filename_component(WORKSPACE_ROOT "${BINDINGS_C_DIR}/../.." ABSOLUTE)

set(HEADER_DIR "${BINDINGS_C_DIR}/include")
set(LIB_DIR "${WORKSPACE_ROOT}/target/debug")

include_directories(${HEADER_DIR})
include_directories(${CMAKE_CURRENT_BINARY_DIR})
include_directories(${CMAKE_CURRENT_SOURCE_DIR})
link_directories(${LIB_DIR})

# Custom command to compile hello_world.idl to hello_world.h using dust_dds_gen
add_custom_command(
    OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/hello_world.h
    COMMAND cargo run --package dust_dds_gen -- ${CMAKE_CURRENT_SOURCE_DIR}/hello_world.idl ${CMAKE_CURRENT_BINARY_DIR}/hello_world.h
    DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/hello_world.idl
    COMMENT "Generating C header from hello_world.idl"
)

# Executables
add_executable(test_create_participant test_create_participant.c)
add_executable(test_datareader_listener test_datareader_listener.c ${CMAKE_CURRENT_BINARY_DIR}/hello_world.h)
add_executable(test_hello_world test_hello_world.c ${CMAKE_CURRENT_BINARY_DIR}/hello_world.h)

# Link libraries
find_package(Threads REQUIRED)
target_link_libraries(test_create_participant PRIVATE unity dust_dds_c Threads::Threads ${CMAKE_DL_LIBS} m)
target_link_libraries(test_datareader_listener PRIVATE unity dust_dds_c Threads::Threads ${CMAKE_DL_LIBS} m)
target_link_libraries(test_hello_world PRIVATE unity dust_dds_c Threads::Threads ${CMAKE_DL_LIBS} m)

# Add CTest tests
add_test(NAME create_participant COMMAND test_create_participant)
add_test(NAME datareader_listener COMMAND test_datareader_listener)
add_test(NAME hello_world COMMAND test_hello_world)
