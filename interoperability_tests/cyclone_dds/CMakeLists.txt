cmake_minimum_required(VERSION 3.25 FATAL_ERROR)

project(DustDDSCycloneDDSInteroperability LANGUAGES C)

# Settings
set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)

# Cyclone DDS
include(FetchContent)
set(ENABLE_SSL OFF)
set(ENABLE_SECURITY OFF)
set(BUILD_TESTING OFF)
set(BUILD_DDSPERF OFF)
set(BUILD_EXAMPLES OFF)
FetchContent_Declare(
  cyclone_dds
  GIT_REPOSITORY https://github.com/eclipse-cyclonedds/cyclonedds.git
  GIT_TAG 7ebcd2c9d928c3781a0de6fbb93a2f98e49a2154 # FIXME
  GIT_SHALLOW ON
  SYSTEM
  FIND_PACKAGE_ARGS 11.0.1 EXACT NAMES CycloneDDS
)
FetchContent_MakeAvailable(cyclone_dds)

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
