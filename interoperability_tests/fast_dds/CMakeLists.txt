cmake_minimum_required(VERSION 3.14.0)
project(FastDdsInteroperability)

find_package(fastcdr REQUIRED)
find_package(fastrtps REQUIRED)
set (CMAKE_CXX_STANDARD 11)

set_property(SOURCE ${CMAKE_BINARY_DIR}/idl/HelloWorld.cxx APPEND PROPERTY GENERATED)
set_property(SOURCE ${CMAKE_BINARY_DIR}/idl/HelloWorldPubSubTypes.cxx APPEND PROPERTY GENERATED)

add_executable(FastDdsPublisher
  fast_dds_publisher.cpp
  ${CMAKE_BINARY_DIR}/idl/HelloWorld.cxx
  ${CMAKE_BINARY_DIR}/idl/HelloWorldPubSubTypes.cxx
)
target_link_libraries(FastDdsPublisher PRIVATE
  fastrtps
  fastcdr
)
target_include_directories(FastDdsPublisher PRIVATE ${CMAKE_BINARY_DIR}/idl)


add_executable(FastDdsSubscriber
  fast_dds_subscriber.cpp
  ${CMAKE_BINARY_DIR}/idl/HelloWorld.cxx
  ${CMAKE_BINARY_DIR}/idl/HelloWorldPubSubTypes.cxx
)
target_link_libraries(FastDdsSubscriber PRIVATE
  fastrtps
  fastcdr
)
target_include_directories(FastDdsSubscriber PRIVATE ${CMAKE_BINARY_DIR}/idl)