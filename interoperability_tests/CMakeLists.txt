cmake_minimum_required(VERSION 3.14.0)
project(Interoperability LANGUAGES C)

find_package(CycloneDDS REQUIRED)
idlc_generate(TARGET GeneratedIdl FILES HelloWorld.idl)

add_executable(CycloneDdsSubscriber
  cyclone_dds_subscriber.c
)
target_link_libraries(CycloneDdsSubscriber PRIVATE
  CycloneDDS::ddsc
  GeneratedIdl
)

add_executable(CycloneDdsPublisher
  cyclone_dds_publisher.c
)
target_link_libraries(CycloneDdsPublisher PRIVATE
  CycloneDDS::ddsc
  GeneratedIdl
)
