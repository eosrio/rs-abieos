# CMAKE generated file: DO NOT EDIT!
# Generated by "Unix Makefiles" Generator, CMake Version 3.24

# Delete rule output on recipe failure.
.DELETE_ON_ERROR:

#=============================================================================
# Special targets provided by cmake.

# Disable implicit rules so canonical targets will work.
.SUFFIXES:

# Disable VCS-based implicit rules.
% : %,v

# Disable VCS-based implicit rules.
% : RCS/%

# Disable VCS-based implicit rules.
% : RCS/%,v

# Disable VCS-based implicit rules.
% : SCCS/s.%

# Disable VCS-based implicit rules.
% : s.%

.SUFFIXES: .hpux_make_needs_suffix_list

# Command-line flag to silence nested $(MAKE).
$(VERBOSE)MAKESILENT = -s

#Suppress display of executed commands.
$(VERBOSE).SILENT:

# A target that is always out of date.
cmake_force:
.PHONY : cmake_force

#=============================================================================
# Set environment variables for the build.

# The shell in which to execute make rules.
SHELL = /bin/sh

# The CMake executable.
CMAKE_COMMAND = /usr/local/bin/cmake

# The command to remove a file.
RM = /usr/local/bin/cmake -E rm -f

# Escaping for special characters.
EQUALS = =

# The top-level source directory on which CMake was run.
CMAKE_SOURCE_DIR = /mnt/c/GitHub/rs-abieos/abieos

# The top-level build directory on which CMake was run.
CMAKE_BINARY_DIR = /mnt/c/GitHub/rs-abieos/abieos

# Include any dependencies generated for this target.
include CMakeFiles/rust_abieos.dir/depend.make
# Include any dependencies generated by the compiler for this target.
include CMakeFiles/rust_abieos.dir/compiler_depend.make

# Include the progress variables for this target.
include CMakeFiles/rust_abieos.dir/progress.make

# Include the compile flags for this target's objects.
include CMakeFiles/rust_abieos.dir/flags.make

CMakeFiles/rust_abieos.dir/src/abieos.cpp.o: CMakeFiles/rust_abieos.dir/flags.make
CMakeFiles/rust_abieos.dir/src/abieos.cpp.o: src/abieos.cpp
CMakeFiles/rust_abieos.dir/src/abieos.cpp.o: CMakeFiles/rust_abieos.dir/compiler_depend.ts
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --progress-dir=/mnt/c/GitHub/rs-abieos/abieos/CMakeFiles --progress-num=$(CMAKE_PROGRESS_1) "Building CXX object CMakeFiles/rust_abieos.dir/src/abieos.cpp.o"
	/usr/bin/c++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -MD -MT CMakeFiles/rust_abieos.dir/src/abieos.cpp.o -MF CMakeFiles/rust_abieos.dir/src/abieos.cpp.o.d -o CMakeFiles/rust_abieos.dir/src/abieos.cpp.o -c /mnt/c/GitHub/rs-abieos/abieos/src/abieos.cpp

CMakeFiles/rust_abieos.dir/src/abieos.cpp.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Preprocessing CXX source to CMakeFiles/rust_abieos.dir/src/abieos.cpp.i"
	/usr/bin/c++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -E /mnt/c/GitHub/rs-abieos/abieos/src/abieos.cpp > CMakeFiles/rust_abieos.dir/src/abieos.cpp.i

CMakeFiles/rust_abieos.dir/src/abieos.cpp.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Compiling CXX source to assembly CMakeFiles/rust_abieos.dir/src/abieos.cpp.s"
	/usr/bin/c++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -S /mnt/c/GitHub/rs-abieos/abieos/src/abieos.cpp -o CMakeFiles/rust_abieos.dir/src/abieos.cpp.s

# Object files for target rust_abieos
rust_abieos_OBJECTS = \
"CMakeFiles/rust_abieos.dir/src/abieos.cpp.o"

# External object files for target rust_abieos
rust_abieos_EXTERNAL_OBJECTS =

librust_abieos.so: CMakeFiles/rust_abieos.dir/src/abieos.cpp.o
librust_abieos.so: CMakeFiles/rust_abieos.dir/build.make
librust_abieos.so: libabieos.a
librust_abieos.so: CMakeFiles/rust_abieos.dir/link.txt
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --bold --progress-dir=/mnt/c/GitHub/rs-abieos/abieos/CMakeFiles --progress-num=$(CMAKE_PROGRESS_2) "Linking CXX shared library librust_abieos.so"
	$(CMAKE_COMMAND) -E cmake_link_script CMakeFiles/rust_abieos.dir/link.txt --verbose=$(VERBOSE)

# Rule to build all files generated by this target.
CMakeFiles/rust_abieos.dir/build: librust_abieos.so
.PHONY : CMakeFiles/rust_abieos.dir/build

CMakeFiles/rust_abieos.dir/clean:
	$(CMAKE_COMMAND) -P CMakeFiles/rust_abieos.dir/cmake_clean.cmake
.PHONY : CMakeFiles/rust_abieos.dir/clean

CMakeFiles/rust_abieos.dir/depend:
	cd /mnt/c/GitHub/rs-abieos/abieos && $(CMAKE_COMMAND) -E cmake_depends "Unix Makefiles" /mnt/c/GitHub/rs-abieos/abieos /mnt/c/GitHub/rs-abieos/abieos /mnt/c/GitHub/rs-abieos/abieos /mnt/c/GitHub/rs-abieos/abieos /mnt/c/GitHub/rs-abieos/abieos/CMakeFiles/rust_abieos.dir/DependInfo.cmake --color=$(COLOR)
.PHONY : CMakeFiles/rust_abieos.dir/depend

