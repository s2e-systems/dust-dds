cmake_minimum_required(VERSION 3.22 FATAL_ERROR)

project(DustDDSCycloneDDSInteroperability LANGUAGES C)

# Settings
set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)

# Cyclone DDS
find_package(CycloneDDS 11.0.1 EXACT QUIET)
if(NOT CycloneDDS_FOUND)
  set(ENABLE_SSL OFF CACHE BOOL "" FORCE)
  set(ENABLE_SECURITY OFF CACHE BOOL "" FORCE)
  set(BUILD_TESTING OFF CACHE BOOL "" FORCE)
  set(BUILD_DDSPERF OFF CACHE BOOL "" FORCE)
  set(BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)

  include(FetchContent)
  FetchContent_Declare(
    CycloneDDS
    GIT_REPOSITORY https://github.com/eclipse-cyclonedds/cyclonedds.git
    # FIXME: proper tag and not a commit hash
    GIT_TAG 2f0d07d241f62f7121749b46721049e4dea5c58b
    # FIXME: uncomment when `GIT_TAG` is a proper tag and not a commit hash,
    # see https://cmake.org/cmake/help/latest/module/ExternalProject.html#id6
    # GIT_SHALLOW ON
    #
    # FIXME: uncomment when `cmake_minimum_required` >= 3.25,
    # see https://cmake.org/cmake/help/latest/module/FetchContent.html#commands
    # SYSTEM
    #
    # FIXME: uncomment when `cmake_minimum_required` >= 3.24,
    # FIND_PACKAGE_ARGS 11.0.1 EXACT
  )
  FetchContent_MakeAvailable(CycloneDDS)
endif()

# Generate subscriber and publisher same idl
function(interoperability_test)
  # Arguments
  cmake_parse_arguments(PARSE_ARGV 0 "" "" "IDL;SUBSCRIBER;PUBLISHER" "")

  # Targets
  cmake_path(GET _IDL STEM LAST_ONLY IDL_TARGET)
  cmake_path(GET _SUBSCRIBER STEM LAST_ONLY SUBSCRIBER_TARGET)
  cmake_path(GET _PUBLISHER STEM LAST_ONLY PUBLISHER_TARGET)

  # IDL
  idlc_generate(TARGET ${IDL_TARGET} FILES "${_IDL}")

  # Subscriber
  add_executable(${SUBSCRIBER_TARGET} "${_SUBSCRIBER}")
  target_link_libraries(${SUBSCRIBER_TARGET} PRIVATE CycloneDDS::ddsc ${IDL_TARGET})
  target_compile_options(
    ${SUBSCRIBER_TARGET}
    PRIVATE
      $<$<C_COMPILER_ID:MSVC>:/W4 /WX>
      $<$<NOT:$<C_COMPILER_ID:MSVC>>:-Wall -Wextra -Wpedantic -Werror>
  )
  if(WIN32)
    # Copy runtime dll(s)
    add_custom_command(
      TARGET ${SUBSCRIBER_TARGET}
      POST_BUILD
      COMMAND
        ${CMAKE_COMMAND} -E
        $<IF:$<BOOL:$<TARGET_RUNTIME_DLLS:${SUBSCRIBER_TARGET}>>,copy_if_different,true>
        $<TARGET_RUNTIME_DLLS:${SUBSCRIBER_TARGET}> $<TARGET_FILE_DIR:${SUBSCRIBER_TARGET}>
      COMMAND_EXPAND_LISTS
      VERBATIM
    )
  endif()

  # Publisher
  add_executable(${PUBLISHER_TARGET} "${_PUBLISHER}")
  target_link_libraries(${PUBLISHER_TARGET} PRIVATE CycloneDDS::ddsc ${IDL_TARGET})
  target_compile_options(
    ${PUBLISHER_TARGET}
    PRIVATE
      $<$<C_COMPILER_ID:MSVC>:/W4 /WX>
      $<$<NOT:$<C_COMPILER_ID:MSVC>>:-Wall -Wextra -Wpedantic -Werror>
  )
  if(WIN32)
    # Copy runtime dll(s)
    add_custom_command(
      TARGET ${PUBLISHER_TARGET}
      POST_BUILD
      COMMAND
        ${CMAKE_COMMAND} -E
        $<IF:$<BOOL:$<TARGET_RUNTIME_DLLS:${PUBLISHER_TARGET}>>,copy_if_different,true>
        $<TARGET_RUNTIME_DLLS:${PUBLISHER_TARGET}> $<TARGET_FILE_DIR:${PUBLISHER_TARGET}>
      COMMAND_EXPAND_LISTS
      VERBATIM
    )
  endif()
endfunction()

# Hello world
interoperability_test(
  IDL ../HelloWorld.idl
  SUBSCRIBER cyclone_dds_subscriber_hello_world.c
  PUBLISHER cyclone_dds_publisher_hello_world.c
)

# Big data
interoperability_test(
  IDL ../BigData.idl
  SUBSCRIBER cyclone_dds_subscriber_big_data.c
  PUBLISHER cyclone_dds_publisher_big_data.c
)

# Dispose
interoperability_test(
  IDL ../DisposeData.idl
  SUBSCRIBER cyclone_dds_subscriber_dispose.c
  PUBLISHER cyclone_dds_publisher_dispose.c
)

# Nested
interoperability_test(
  IDL ../NestedType.idl
  SUBSCRIBER cyclone_dds_subscriber_nested.c
  PUBLISHER cyclone_dds_publisher_nested.c
)

# Union
interoperability_test(
  IDL ../Union.idl
  SUBSCRIBER cyclone_dds_subscriber_union.c
  PUBLISHER cyclone_dds_publisher_union.c
)

# Inheritance
interoperability_test(
  IDL ../Inheritance.idl
  SUBSCRIBER cyclone_dds_subscriber_inheritance.c
  PUBLISHER cyclone_dds_publisher_inheritance.c
)

# Optional
interoperability_test(
  IDL ../Optional.idl
  SUBSCRIBER cyclone_dds_subscriber_optional.c
  PUBLISHER cyclone_dds_publisher_optional.c
)
