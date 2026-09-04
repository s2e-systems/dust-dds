cmake_minimum_required(VERSION 3.14.0)
project(FastDdsInteroperability)

find_package(fastcdr REQUIRED)
find_package(fastdds REQUIRED)
set (CMAKE_CXX_STANDARD 11)
add_compile_options(-Werror -Wall)

set_source_files_properties(${CMAKE_BINARY_DIR}/idl/interoperability_tests/HelloWorldTypeObjectSupport.cxx PROPERTIES GENERATED TRUE)
set_source_files_properties(${CMAKE_BINARY_DIR}/idl/interoperability_tests/HelloWorldPubSubTypes.cxx PROPERTIES GENERATED TRUE)
set_source_files_properties(${CMAKE_BINARY_DIR}/idl/interoperability_tests/DisposeDataTypeObjectSupport.cxx PROPERTIES GENERATED TRUE)
set_source_files_properties(${CMAKE_BINARY_DIR}/idl/interoperability_tests/DisposeDataPubSubTypes.cxx PROPERTIES GENERATED TRUE)

add_executable(FastDdsPublisher
  fast_dds_publisher.cpp
  ${CMAKE_BINARY_DIR}/idl/interoperability_tests/HelloWorldTypeObjectSupport.cxx
  ${CMAKE_BINARY_DIR}/idl/interoperability_tests/HelloWorldPubSubTypes.cxx
)
target_link_libraries(FastDdsPublisher PRIVATE
  fastdds
  fastcdr
)
target_include_directories(FastDdsPublisher PRIVATE ${CMAKE_BINARY_DIR}/idl/interoperability_tests)


add_executable(FastDdsSubscriber
  fast_dds_subscriber.cpp
  ${CMAKE_BINARY_DIR}/idl/interoperability_tests/HelloWorldTypeObjectSupport.cxx
  ${CMAKE_BINARY_DIR}/idl/interoperability_tests/HelloWorldPubSubTypes.cxx
)
target_link_libraries(FastDdsSubscriber PRIVATE
  fastdds
  fastcdr
)
target_include_directories(FastDdsSubscriber PRIVATE ${CMAKE_BINARY_DIR}/idl/interoperability_tests)

add_executable(FastDdsPublisherDispose
  fast_dds_publisher_dispose.cpp
  ${CMAKE_BINARY_DIR}/idl/interoperability_tests/DisposeDataTypeObjectSupport.cxx
  ${CMAKE_BINARY_DIR}/idl/interoperability_tests/DisposeDataPubSubTypes.cxx
)
target_link_libraries(FastDdsPublisherDispose PRIVATE
  fastdds
  fastcdr
)
target_include_directories(FastDdsPublisherDispose PRIVATE ${CMAKE_BINARY_DIR}/idl/interoperability_tests)

add_executable(FastDdsSubscriberDispose
  fast_dds_subscriber_dispose.cpp
  ${CMAKE_BINARY_DIR}/idl/interoperability_tests/DisposeDataTypeObjectSupport.cxx
  ${CMAKE_BINARY_DIR}/idl/interoperability_tests/DisposeDataPubSubTypes.cxx
)
target_link_libraries(FastDdsSubscriberDispose PRIVATE
  fastdds
  fastcdr
)
target_include_directories(FastDdsSubscriberDispose PRIVATE ${CMAKE_BINARY_DIR}/idl/interoperability_tests)