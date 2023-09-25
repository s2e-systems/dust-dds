cmake_minimum_required(VERSION 3.14.0)
project(Interoperability)

find_package(fastcdr REQUIRED)
find_package(fastrtps REQUIRED)
set (CMAKE_CXX_STANDARD 11)

add_executable(FastDdsSubscriber
  fast_dds_subscriber.cpp
  build/idl/HelloWorld.cxx
  build/idl/HelloWorldPubSubTypes.cxx
)
target_link_libraries(FastDdsSubscriber PRIVATE
  fastrtps
  fastcdr
)

target_include_directories(FastDdsSubscriber PRIVATE build/idl)