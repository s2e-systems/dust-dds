cmake_minimum_required(VERSION 3.14.0)
project(Compatibility LANGUAGES CXX C)

find_package(CycloneDDS REQUIRED)
idlc_generate(TARGET GeneratedIdl FILES HelloWorld.idl)

find_package(CycloneDDS-CXX REQUIRED)
idlcxx_generate(TARGET GeneratedIdlCXX FILES HelloWorld.idl)

add_executable(CompatibilitySubscriber
  subscriber/main.cpp
)
target_link_libraries(CompatibilitySubscriber PRIVATE
  CycloneDDS::ddsc
  GeneratedIdl
  CycloneDDS-CXX::ddscxx
  GeneratedIdlCXX
)


add_executable(CompatibilityPublisher
  publisher/main.c
)
target_link_libraries(CompatibilityPublisher PRIVATE
  CycloneDDS::ddsc
  GeneratedIdl
)
