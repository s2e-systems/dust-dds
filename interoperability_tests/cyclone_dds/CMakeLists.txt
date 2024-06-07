cmake_minimum_required(VERSION 3.14.0)
project(CycloneInteroperability LANGUAGES C)

find_package(CycloneDDS REQUIRED)
idlc_generate(TARGET HelloWorldGeneratedIdl FILES ../HelloWorld.idl)
idlc_generate(TARGET BigDataGeneratedIdl FILES ../BigData.idl)
idlc_generate(TARGET DisposeDataGeneratedIdl FILES ../DisposeData.idl)
idlc_generate(TARGET NestedTypeGeneratedIdl FILES ../NestedType.idl)

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