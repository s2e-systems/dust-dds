cmake_minimum_required(VERSION 3.14.0)
project(Interoperability LANGUAGES C)

find_package(CycloneDDS REQUIRED)
idlc_generate(TARGET GeneratedIdl FILES HelloWorld.idl)

add_executable(InteroperabilitySubscriber
  cyclone_dds_subscriber.c
)
target_link_libraries(InteroperabilitySubscriber PRIVATE
  CycloneDDS::ddsc
  GeneratedIdl
)

add_executable(InteroperabilityPublisher
  cyclone_dds_publisher.c
)
target_link_libraries(InteroperabilityPublisher PRIVATE
  CycloneDDS::ddsc
  GeneratedIdl
)
