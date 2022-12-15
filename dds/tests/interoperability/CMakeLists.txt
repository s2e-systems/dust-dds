cmake_minimum_required(VERSION 3.14.0)
project(Compatibility LANGUAGES C)

find_package(CycloneDDS REQUIRED)
idlc_generate(TARGET GeneratedIdl FILES HelloWorld.idl)

add_executable(CompatibilitySubscriber
  subscriber/main.c
)
target_link_libraries(CompatibilitySubscriber PRIVATE
  CycloneDDS::ddsc
  GeneratedIdl
)

add_executable(CompatibilityPublisher
  publisher/main.c
)
target_link_libraries(CompatibilityPublisher PRIVATE
  CycloneDDS::ddsc
  GeneratedIdl
)
