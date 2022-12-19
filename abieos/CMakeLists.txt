cmake_minimum_required (VERSION 3.24)
project(rustabieos VERSION 2.2 LANGUAGES CXX C)

set(default_build_type "Release")
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS ON)

option(ABIEOS_NO_INT128 "disable use of __int128" OFF)

find_package(Threads)

add_library(abieos STATIC src/abi.cpp src/crypto.cpp include/eosio/fpconv.c)
target_include_directories(abieos PUBLIC include)

if(ABIEOS_NO_INT128)
target_compile_definitions(abieos PUBLIC ABIEOS_NO_INT128)
endif()

set_target_properties(abieos PROPERTIES POSITION_INDEPENDENT_CODE ON)

add_library(${PROJECT_NAME} SHARED src/abieos.cpp src/abieos.h)
target_link_libraries(${PROJECT_NAME} abieos;${CMAKE_THREAD_LIBS_INIT})
target_compile_options(${PROJECT_NAME} PRIVATE -Wall -Wextra -Wno-unused-parameter)
