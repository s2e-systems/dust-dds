cmake_minimum_required(VERSION 3.22 FATAL_ERROR)

project(cyclone_dds_interoperability LANGUAGES C)

#
# Settings
#
set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(BUILD_SHARED_LIBS OFF)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)
get_property(_generator_is_multi_config GLOBAL PROPERTY GENERATOR_IS_MULTI_CONFIG)
if(_generator_is_multi_config)
  set(CMAKE_CONFIGURATION_TYPES Debug)
  if(CMAKE_GENERATOR STREQUAL "Ninja Multi-Config")
    set(CMAKE_DEFAULT_BUILD_TYPE Debug)
  endif()
else()
  set(CMAKE_BUILD_TYPE Debug)
endif()

#
# Cyclone DDS
#
set(ENABLE_SSL OFF CACHE BOOL "" FORCE)
set(ENABLE_SECURITY OFF CACHE BOOL "" FORCE)
set(BUILD_TESTING OFF CACHE BOOL "" FORCE)
set(BUILD_DDSPERF OFF CACHE BOOL "" FORCE)
set(BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)

include(FetchContent)
FetchContent_Declare(
    CycloneDDS
    GIT_REPOSITORY https://github.com/eclipse-cyclonedds/cyclonedds.git
    # TODO: proper tag and not a commit hash,
    #       latest known commit with all CI checks passing
    GIT_TAG 663cdbee019e06d97222463b72220b9061170ab7
    # TODO: uncomment when `GIT_TAG` is a proper tag and not a commit hash,
    # see https://cmake.org/cmake/help/latest/module/ExternalProject.html#id6
    # GIT_SHALLOW ON
    #
    # TODO: uncomment when `cmake_minimum_required` >= 3.25,
    # see https://cmake.org/cmake/help/latest/module/FetchContent.html#commands
    # SYSTEM
  )
FetchContent_MakeAvailable(CycloneDDS)

#
# Compile options
#
if (MSVC)
  add_compile_options(/W4 /WX)
elseif(CMAKE_C_COMPILER_ID MATCHES "GNU|Clang")
  add_compile_options(-Wall -Wextra -Wpedantic -Werror)
endif()

#
# Hello world
#
# IDL
idlc_generate(TARGET hello_world_idl FILES ../HelloWorld.idl)
# Subscriber
add_executable(cyclone_dds_subscriber_hello_world
  cyclone_dds_subscriber_hello_world.c
)
target_link_libraries(cyclone_dds_subscriber_hello_world PRIVATE
  CycloneDDS::ddsc
  hello_world_idl
)
# Publisher
add_executable(cyclone_dds_publisher_hello_world
  cyclone_dds_publisher_hello_world.c
)
target_link_libraries(cyclone_dds_publisher_hello_world PRIVATE
  CycloneDDS::ddsc
  hello_world_idl
)

#
# Big data
#
# IDL
idlc_generate(TARGET big_data_idl FILES ../BigData.idl)
# Subscriber
add_executable(cyclone_dds_subscriber_big_data
  cyclone_dds_subscriber_big_data.c
)
target_link_libraries(cyclone_dds_subscriber_big_data PRIVATE
  CycloneDDS::ddsc
  big_data_idl
)
# Publisher
add_executable(cyclone_dds_publisher_big_data
  cyclone_dds_publisher_big_data.c
)
target_link_libraries(cyclone_dds_publisher_big_data PRIVATE
  CycloneDDS::ddsc
  big_data_idl
)

#
# Dispose
#
# IDL
idlc_generate(TARGET dispose_data_idl FILES ../DisposeData.idl)
# Subscriber
add_executable(cyclone_dds_subscriber_dispose
  cyclone_dds_subscriber_dispose.c
)
target_link_libraries(cyclone_dds_subscriber_dispose PRIVATE
  CycloneDDS::ddsc
  dispose_data_idl
)
# Publisher
add_executable(cyclone_dds_publisher_dispose
  cyclone_dds_publisher_dispose.c
)
target_link_libraries(cyclone_dds_publisher_dispose PRIVATE
  CycloneDDS::ddsc
  dispose_data_idl
)

#
# Nested
#
# IDL
idlc_generate(TARGET nested_type_idl FILES ../NestedType.idl)
# Subscriber
add_executable(cyclone_dds_subscriber_nested
  cyclone_dds_subscriber_nested.c
)
target_link_libraries(cyclone_dds_subscriber_nested PRIVATE
  CycloneDDS::ddsc
  nested_type_idl
)
# Publisher
add_executable(cyclone_dds_publisher_nested
  cyclone_dds_publisher_nested.c
)
target_link_libraries(cyclone_dds_publisher_nested PRIVATE
  CycloneDDS::ddsc
  nested_type_idl
)

#
# Union
#
# IDL
idlc_generate(TARGET union_idl FILES ../Union.idl)
# Subscriber
add_executable(cyclone_dds_subscriber_union
  cyclone_dds_subscriber_union.c
)
target_link_libraries(cyclone_dds_subscriber_union PRIVATE
  CycloneDDS::ddsc
  union_idl
)
# Publisher
add_executable(cyclone_dds_publisher_union
  cyclone_dds_publisher_union.c
)
target_link_libraries(cyclone_dds_publisher_union PRIVATE
  CycloneDDS::ddsc
  union_idl
)

#
# Inheritance
#
# IDL
idlc_generate(TARGET inheritance_idl FILES ../Inheritance.idl)
# Subscriber
add_executable(cyclone_dds_subscriber_inheritance
  cyclone_dds_subscriber_inheritance.c
)
target_link_libraries(cyclone_dds_subscriber_inheritance PRIVATE
  CycloneDDS::ddsc
  inheritance_idl
)
# Publisher
add_executable(cyclone_dds_publisher_inheritance
  cyclone_dds_publisher_inheritance.c
)
target_link_libraries(cyclone_dds_publisher_inheritance PRIVATE
  CycloneDDS::ddsc
  inheritance_idl
)

#
# Optional
#
# IDL
idlc_generate(TARGET optional_idl FILES ../Optional.idl)
# Subscriber
add_executable(cyclone_dds_subscriber_optional
  cyclone_dds_subscriber_optional.c
)
target_link_libraries(cyclone_dds_subscriber_optional PRIVATE
  CycloneDDS::ddsc
  optional_idl
)
# Publisher
add_executable(cyclone_dds_publisher_optional
  cyclone_dds_publisher_optional.c
)
target_link_libraries(cyclone_dds_publisher_optional PRIVATE
  CycloneDDS::ddsc
  optional_idl
)
