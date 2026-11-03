cmake_minimum_required(VERSION 3.16.0)
project(CycloneInteroperability LANGUAGES C)

include(FetchContent)
FetchContent_Declare(
  CycloneDDS
  GIT_REPOSITORY https://github.com/eclipse-cyclonedds/cyclonedds.git
  GIT_TAG 3a5526315e64ac1d2a52e6ac1ae7f3e861c593d3
  GIT_SHALLOW TRUE
)
set(CMAKE_BUILD_TYPE Debug CACHE STRING "" FORCE)
set(BUILD_TESTING OFF CACHE BOOL "" FORCE)
set(BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
set(BUILD_DDSPERF OFF CACHE BOOL "" FORCE)
set(ENABLE_SSL OFF CACHE STRING "" FORCE)
set(ENABLE_SECURITY OFF CACHE BOOL "" FORCE)
set(CMAKE_PROJECT_NAME CycloneDDS)
FetchContent_MakeAvailable(CycloneDDS)

add_compile_options(-Werror -Wall)
idlc_generate(TARGET HelloWorldGeneratedIdl FILES ../HelloWorld.idl)
idlc_generate(TARGET BigDataGeneratedIdl FILES ../BigData.idl)
idlc_generate(TARGET DisposeDataGeneratedIdl FILES ../DisposeData.idl)
idlc_generate(TARGET NestedTypeGeneratedIdl FILES ../NestedType.idl)
idlc_generate(TARGET UnionGeneratedIdl FILES ../Union.idl)
idlc_generate(TARGET InheritanceGeneratedIdl FILES ../Inheritance.idl)
idlc_generate(TARGET OptionalGeneratedIdl FILES ../Optional.idl)

add_executable(CycloneDdsSubscriber
  cyclone_dds_subscriber.c
)
target_link_libraries(CycloneDdsSubscriber PRIVATE
  CycloneDDS::ddsc
  HelloWorldGeneratedIdl
)

add_executable(CycloneDdsPublisher
  cyclone_dds_publisher.c
)
target_link_libraries(CycloneDdsPublisher PRIVATE
  CycloneDDS::ddsc
  HelloWorldGeneratedIdl
)


add_executable(CycloneDdsBigDataSubscriber
  cyclone_dds_big_data_subscriber.c
)
target_link_libraries(CycloneDdsBigDataSubscriber PRIVATE
  CycloneDDS::ddsc
  BigDataGeneratedIdl
)

add_executable(CycloneDdsBigDataPublisher
  cyclone_dds_big_data_publisher.c
)
target_link_libraries(CycloneDdsBigDataPublisher PRIVATE
  CycloneDDS::ddsc
  BigDataGeneratedIdl
)


add_executable(CycloneDdsSubscriberDispose
  cyclone_dds_subscriber_dispose.c
)
target_link_libraries(CycloneDdsSubscriberDispose PRIVATE
  CycloneDDS::ddsc
  DisposeDataGeneratedIdl
)

add_executable(CycloneDdsPublisherDispose
  cyclone_dds_publisher_dispose.c
)
target_link_libraries(CycloneDdsPublisherDispose PRIVATE
  CycloneDDS::ddsc
  DisposeDataGeneratedIdl
)

add_executable(CycloneDdsPublisherNested
  cyclone_dds_publisher_nested.c
)
target_link_libraries(CycloneDdsPublisherNested PRIVATE
  CycloneDDS::ddsc
  NestedTypeGeneratedIdl
)

add_executable(CycloneDdsSubscriberNested
  cyclone_dds_subscriber_nested.c
)
target_link_libraries(CycloneDdsSubscriberNested PRIVATE
  CycloneDDS::ddsc
  NestedTypeGeneratedIdl
)

add_executable(CycloneDdsPublisherUnion
  cyclone_dds_publisher_union.c
)
target_link_libraries(CycloneDdsPublisherUnion PRIVATE
  CycloneDDS::ddsc
  UnionGeneratedIdl
)

add_executable(CycloneDdsSubscriberUnion
  cyclone_dds_subscriber_union.c
)
target_link_libraries(CycloneDdsSubscriberUnion PRIVATE
  CycloneDDS::ddsc
  UnionGeneratedIdl
)

add_executable(CycloneDdsSubscriberInheritance
  cyclone_dds_subscriber_inheritance.c
)
target_link_libraries(CycloneDdsSubscriberInheritance PRIVATE
  CycloneDDS::ddsc
  InheritanceGeneratedIdl
)

add_executable(CycloneDdsPublisherInheritance
  cyclone_dds_publisher_inheritance.c
)
target_link_libraries(CycloneDdsPublisherInheritance PRIVATE
  CycloneDDS::ddsc
  InheritanceGeneratedIdl
)

add_executable(CycloneDdsSubscriberOptional
  cyclone_dds_subscriber_optional.c
)
target_link_libraries(CycloneDdsSubscriberOptional PRIVATE
  CycloneDDS::ddsc
  OptionalGeneratedIdl
)

add_executable(CycloneDdsPublisherOptional
  cyclone_dds_publisher_optional.c
)
target_link_libraries(CycloneDdsPublisherOptional PRIVATE
  CycloneDDS::ddsc
  OptionalGeneratedIdl
)
