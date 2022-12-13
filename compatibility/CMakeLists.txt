cmake_minimum_required(VERSION 3.14.0)
project(Compatibility LANGUAGES CXX)


find_package(CycloneDDS-CXX REQUIRED)
idlcxx_generate(TARGET GeneratedIdl FILES HelloWorld.idl)

add_executable(CompatibilitySubscriber
  subscriber/main.cpp
)
target_include_directories(CompatibilitySubscriber PRIVATE
  CompatibilitySubscriber
)
target_link_libraries(CompatibilitySubscriber PRIVATE
  CycloneDDS-CXX::ddscxx
  GeneratedIdl
)
target_compile_features(CompatibilitySubscriber PRIVATE cxx_std_17)


add_executable(CompatibilityPublisher
  publisher/main.cpp
)
target_include_directories(CompatibilityPublisher PRIVATE
  CompatibilityPublisher
  GeneratedIdl
)
target_link_libraries(CompatibilityPublisher PRIVATE
  CycloneDDS-CXX::ddscxx
  GeneratedIdl
)
target_compile_features(CompatibilitySubscriber PRIVATE cxx_std_17)
