cmake_minimum_required(VERSION 3.14.0)
project(Interoperability LANGUAGES C)

find_package(CycloneDDS REQUIRED)
idlc_generate(TARGET HelloWorldGeneratedIdl FILES HelloWorld.idl)
idlc_generate(TARGET BigDataGeneratedIdl FILES BigData.idl)

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
