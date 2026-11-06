cmake_minimum_required(VERSION 3.22.0)
project(FastDdsInteroperability)

include(FetchContent)

set(BUILD_SHARED_LIBS ON CACHE BOOL "" FORCE)
set(BUILD_TESTING OFF CACHE BOOL "" FORCE)

set(FOONATHAN_MEMORY_BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
set(FOONATHAN_MEMORY_BUILD_TESTS OFF CACHE BOOL "" FORCE)
set(FOONATHAN_MEMORY_BUILD_TOOLS ON CACHE BOOL "" FORCE)
FetchContent_Declare(
  foonathan_memory
  GIT_REPOSITORY https://github.com/eProsima/memory.git
  GIT_TAG vendor-1.4.1
  GIT_SHALLOW TRUE
)

FetchContent_Declare(
  fastcdr
  GIT_REPOSITORY https://github.com/eProsima/Fast-CDR.git
  GIT_TAG v2.4.0
  GIT_SHALLOW TRUE
)

set(THIRDPARTY_Asio ON CACHE STRING "" FORCE)
set(THIRDPARTY_TinyXML2 ON CACHE STRING "" FORCE)
set(COMPILE_TOOLS OFF CACHE BOOL "" FORCE)
FetchContent_Declare(
  fastdds
  GIT_REPOSITORY https://github.com/eProsima/Fast-DDS.git
  GIT_TAG v3.6.2
  GIT_SHALLOW TRUE
  GIT_SUBMODULES ""
)

FetchContent_Declare(
  fastddsgen
  GIT_REPOSITORY https://github.com/eProsima/Fast-DDS-Gen.git
  GIT_TAG v4.3.0
  GIT_SHALLOW TRUE
)

file(WRITE "${CMAKE_CURRENT_BINARY_DIR}/cmake/Findfoonathan_memory.cmake" "set(foonathan_memory_FOUND TRUE)\n")
file(WRITE "${CMAKE_CURRENT_BINARY_DIR}/cmake/Findfastcdr.cmake" "set(fastcdr_FOUND TRUE)\n")
list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_BINARY_DIR}/cmake")

FetchContent_MakeAvailable(foonathan_memory fastcdr fastdds fastddsgen)

set(CMAKE_CXX_STANDARD 11)
add_compile_options(-Werror -Wall)

set(FASTDDSGEN_JAR "${fastddsgen_SOURCE_DIR}/share/fastddsgen/java/fastddsgen.jar")
add_custom_command(
  OUTPUT ${FASTDDSGEN_JAR}
  COMMAND ./gradlew assemble
  WORKING_DIRECTORY ${fastddsgen_SOURCE_DIR}
  COMMENT "Building Fast-DDS-Gen"
)
add_custom_target(fastddsgen_build DEPENDS ${FASTDDSGEN_JAR})

set(IDL_OUTPUT_DIR "${CMAKE_CURRENT_BINARY_DIR}/idl")
file(MAKE_DIRECTORY ${IDL_OUTPUT_DIR})

set(HELLOWORLD_GENERATED_SOURCES
  ${IDL_OUTPUT_DIR}/HelloWorldTypeObjectSupport.cxx
  ${IDL_OUTPUT_DIR}/HelloWorldPubSubTypes.cxx
)
add_custom_command(
  OUTPUT
    ${HELLOWORLD_GENERATED_SOURCES}
    ${IDL_OUTPUT_DIR}/HelloWorld.hpp
    ${IDL_OUTPUT_DIR}/HelloWorldPubSubTypes.hpp
    ${IDL_OUTPUT_DIR}/HelloWorldTypeObjectSupport.hpp
  COMMAND java -jar ${FASTDDSGEN_JAR} -replace -d ${IDL_OUTPUT_DIR} -flat-output-dir ${CMAKE_CURRENT_SOURCE_DIR}/../HelloWorld.idl
  DEPENDS ${FASTDDSGEN_JAR} fastddsgen_build ${CMAKE_CURRENT_SOURCE_DIR}/../HelloWorld.idl
  COMMENT "Generating FastDDS code for HelloWorld.idl"
)
add_custom_target(HelloWorldGeneratedIdl DEPENDS ${HELLOWORLD_GENERATED_SOURCES})

set(DISPOSEDATA_GENERATED_SOURCES
  ${IDL_OUTPUT_DIR}/DisposeDataTypeObjectSupport.cxx
  ${IDL_OUTPUT_DIR}/DisposeDataPubSubTypes.cxx
)
add_custom_command(
  OUTPUT
    ${DISPOSEDATA_GENERATED_SOURCES}
    ${IDL_OUTPUT_DIR}/DisposeData.hpp
    ${IDL_OUTPUT_DIR}/DisposeDataPubSubTypes.hpp
    ${IDL_OUTPUT_DIR}/DisposeDataTypeObjectSupport.hpp
  COMMAND java -jar ${FASTDDSGEN_JAR} -replace -d ${IDL_OUTPUT_DIR} -flat-output-dir ${CMAKE_CURRENT_SOURCE_DIR}/../DisposeData.idl
  DEPENDS ${FASTDDSGEN_JAR} fastddsgen_build ${CMAKE_CURRENT_SOURCE_DIR}/../DisposeData.idl
  COMMENT "Generating FastDDS code for DisposeData.idl"
)
add_custom_target(DisposeDataGeneratedIdl DEPENDS ${DISPOSEDATA_GENERATED_SOURCES})

add_executable(FastDdsPublisher
  fast_dds_publisher.cpp
  ${HELLOWORLD_GENERATED_SOURCES}
)
add_dependencies(FastDdsPublisher HelloWorldGeneratedIdl)
target_link_libraries(FastDdsPublisher PRIVATE
  fastdds
  fastcdr
)
target_include_directories(FastDdsPublisher PRIVATE ${IDL_OUTPUT_DIR})


add_executable(FastDdsSubscriber
  fast_dds_subscriber.cpp
  ${HELLOWORLD_GENERATED_SOURCES}
)
add_dependencies(FastDdsSubscriber HelloWorldGeneratedIdl)
target_link_libraries(FastDdsSubscriber PRIVATE
  fastdds
  fastcdr
)
target_include_directories(FastDdsSubscriber PRIVATE ${IDL_OUTPUT_DIR})

add_executable(FastDdsPublisherDispose
  fast_dds_publisher_dispose.cpp
  ${DISPOSEDATA_GENERATED_SOURCES}
)
add_dependencies(FastDdsPublisherDispose DisposeDataGeneratedIdl)
target_link_libraries(FastDdsPublisherDispose PRIVATE
  fastdds
  fastcdr
)
target_include_directories(FastDdsPublisherDispose PRIVATE ${IDL_OUTPUT_DIR})

add_executable(FastDdsSubscriberDispose
  fast_dds_subscriber_dispose.cpp
  ${DISPOSEDATA_GENERATED_SOURCES}
)
add_dependencies(FastDdsSubscriberDispose DisposeDataGeneratedIdl)
target_link_libraries(FastDdsSubscriberDispose PRIVATE
  fastdds
  fastcdr
)
target_include_directories(FastDdsSubscriberDispose PRIVATE ${IDL_OUTPUT_DIR})