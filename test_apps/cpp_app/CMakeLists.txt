# CMakeLists.txt for C++ Test Application
# Demonstrates various C++ language constructs for the EMBARGO framework

cmake_minimum_required(VERSION 3.16)

# Project information
project(CppTestApp
    VERSION 1.0.0
    DESCRIPTION "C++ Test Application for EMBARGO Framework"
    LANGUAGES CXX
)

# Set C++ standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Set default build type to Release if not specified
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release CACHE STRING "Choose the type of build." FORCE)
    set_property(CACHE CMAKE_BUILD_TYPE PROPERTY STRINGS "Debug" "Release" "MinSizeRel" "RelWithDebInfo")
endif()

# Compiler-specific options
if(CMAKE_CXX_COMPILER_ID STREQUAL "GNU" OR CMAKE_CXX_COMPILER_ID STREQUAL "Clang")
    set(CMAKE_CXX_FLAGS_DEBUG "-g -O0 -DDEBUG -Wall -Wextra -Wpedantic")
    set(CMAKE_CXX_FLAGS_RELEASE "-O3 -DNDEBUG -Wall -Wextra")
    set(CMAKE_CXX_FLAGS_RELWITHDEBINFO "-O2 -g -DNDEBUG -Wall -Wextra")
    set(CMAKE_CXX_FLAGS_MINSIZEREL "-Os -DNDEBUG")
elseif(CMAKE_CXX_COMPILER_ID STREQUAL "MSVC")
    set(CMAKE_CXX_FLAGS_DEBUG "/Od /Zi /DDEBUG /W4")
    set(CMAKE_CXX_FLAGS_RELEASE "/O2 /DNDEBUG /W3")
endif()

# Include directories
include_directories(include)

# Find required packages
find_package(Threads REQUIRED)

# Source files
file(GLOB_RECURSE SOURCES "src/*.cpp")
file(GLOB_RECURSE HEADERS "include/*.hpp")

# Create executable
add_executable(${PROJECT_NAME} ${SOURCES} ${HEADERS})

# Link libraries
target_link_libraries(${PROJECT_NAME} 
    PRIVATE 
    Threads::Threads
)

# Set target properties
set_target_properties(${PROJECT_NAME} PROPERTIES
    OUTPUT_NAME "cpp_test_app"
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/bin"
)

# Enable additional warnings
if(CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
    target_compile_options(${PROJECT_NAME} PRIVATE
        -Wall
        -Wextra
        -Wpedantic
        -Wcast-align
        -Wcast-qual
        -Wconversion
        -Wctor-dtor-privacy
        -Wduplicated-cond
        -Wduplicated-branches
        -Wfloat-equal
        -Wlogical-op
        -Wnon-virtual-dtor
        -Wold-style-cast
        -Woverloaded-virtual
        -Wredundant-decls
        -Wsign-conversion
        -Wsign-promo
    )
elseif(CMAKE_CXX_COMPILER_ID STREQUAL "Clang")
    target_compile_options(${PROJECT_NAME} PRIVATE
        -Wall
        -Wextra
        -Wpedantic
        -Wcast-align
        -Wcast-qual
        -Wconversion
        -Wctor-dtor-privacy
        -Wfloat-equal
        -Wnon-virtual-dtor
        -Wold-style-cast
        -Woverloaded-virtual
        -Wredundant-decls
        -Wsign-conversion
        -Wsign-promo
    )
endif()

# Add compile definitions
target_compile_definitions(${PROJECT_NAME} PRIVATE
    APP_VERSION_MAJOR=${PROJECT_VERSION_MAJOR}
    APP_VERSION_MINOR=${PROJECT_VERSION_MINOR}
    APP_VERSION_PATCH=${PROJECT_VERSION_PATCH}
    APP_VERSION_STRING="${PROJECT_VERSION}"
)

# Installation
install(TARGETS ${PROJECT_NAME}
    RUNTIME DESTINATION bin
)

# Create default config file
configure_file(
    "${CMAKE_SOURCE_DIR}/config.json.in"
    "${CMAKE_BINARY_DIR}/config.json"
    @ONLY
)

# Custom targets

# Format target (requires clang-format)
find_program(CLANG_FORMAT "clang-format")
if(CLANG_FORMAT)
    add_custom_target(format
        COMMAND ${CLANG_FORMAT} -i ${SOURCES} ${HEADERS}
        COMMENT "Formatting code with clang-format"
        VERBATIM
    )
endif()

# Static analysis target (requires cppcheck)
find_program(CPPCHECK "cppcheck")
if(CPPCHECK)
    add_custom_target(analyze
        COMMAND ${CPPCHECK} --enable=all --inconclusive --std=c++17 
                --project=${CMAKE_BINARY_DIR}/compile_commands.json
                --suppress=missingIncludeSystem
        COMMENT "Running static analysis with cppcheck"
        VERBATIM
    )
endif()

# Documentation target (requires doxygen)
find_package(Doxygen)
if(DOXYGEN_FOUND)
    set(DOXYGEN_IN ${CMAKE_CURRENT_SOURCE_DIR}/docs/Doxyfile.in)
    set(DOXYGEN_OUT ${CMAKE_CURRENT_BINARY_DIR}/Doxyfile)
    
    if(EXISTS ${DOXYGEN_IN})
        configure_file(${DOXYGEN_IN} ${DOXYGEN_OUT} @ONLY)
        
        add_custom_target(docs
            COMMAND ${DOXYGEN_EXECUTABLE} ${DOXYGEN_OUT}
            WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
            COMMENT "Generating API documentation with Doxygen"
            VERBATIM
        )
    endif()
endif()

# Valgrind target (Linux only)
if(UNIX AND NOT APPLE)
    find_program(VALGRIND "valgrind")
    if(VALGRIND)
        add_custom_target(valgrind
            COMMAND ${VALGRIND} --leak-check=full --show-leak-kinds=all 
                    --track-origins=yes $<TARGET_FILE:${PROJECT_NAME}>
            DEPENDS ${PROJECT_NAME}
            COMMENT "Running valgrind memory check"
            VERBATIM
        )
    endif()
endif()

# Test target
add_custom_target(test
    COMMAND $<TARGET_FILE:${PROJECT_NAME}>
    DEPENDS ${PROJECT_NAME}
    COMMENT "Running the application"
    VERBATIM
)

# Quick test with timeout
add_custom_target(quick-test
    COMMAND timeout 10s $<TARGET_FILE:${PROJECT_NAME}> || true
    DEPENDS ${PROJECT_NAME}
    COMMENT "Running quick functionality test"
    VERBATIM
)

# Print build information
add_custom_target(info
    COMMAND ${CMAKE_COMMAND} -E echo "=== Build Information ==="
    COMMAND ${CMAKE_COMMAND} -E echo "Project: ${PROJECT_NAME}"
    COMMAND ${CMAKE_COMMAND} -E echo "Version: ${PROJECT_VERSION}"
    COMMAND ${CMAKE_COMMAND} -E echo "Build type: ${CMAKE_BUILD_TYPE}"
    COMMAND ${CMAKE_COMMAND} -E echo "Compiler: ${CMAKE_CXX_COMPILER_ID} ${CMAKE_CXX_COMPILER_VERSION}"
    COMMAND ${CMAKE_COMMAND} -E echo "C++ standard: ${CMAKE_CXX_STANDARD}"
    COMMAND ${CMAKE_COMMAND} -E echo "Binary dir: ${CMAKE_BINARY_DIR}"
    COMMAND ${CMAKE_COMMAND} -E echo "Source dir: ${CMAKE_SOURCE_DIR}"
    COMMAND ${CMAKE_COMMAND} -E echo "========================="
    COMMENT "Displaying build information"
    VERBATIM
)

# Enable compile_commands.json for IDE support
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# CPack configuration for packaging
include(CPack)
set(CPACK_PACKAGE_NAME "CppTestApp")
set(CPACK_PACKAGE_VERSION ${PROJECT_VERSION})
set(CPACK_PACKAGE_DESCRIPTION_SUMMARY ${PROJECT_DESCRIPTION})
set(CPACK_PACKAGE_VENDOR "EMBARGO Framework")
set(CPACK_PACKAGE_CONTACT "embargo@example.com")

if(WIN32)
    set(CPACK_GENERATOR "NSIS;ZIP")
elseif(APPLE)
    set(CPACK_GENERATOR "DragNDrop;TGZ")
else()
    set(CPACK_GENERATOR "DEB;RPM;TGZ")
endif()
