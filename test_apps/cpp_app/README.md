# C++ Test Application for EMBARGO Framework

This C++ application demonstrates various C++ language constructs and serves as a test case for the EMBARGO dependency analysis framework.

## Features Demonstrated

### Object-Oriented Programming
- **Inheritance**: `User` → `AdminUser` inheritance hierarchy
- **Polymorphism**: Virtual functions and dynamic dispatch
- **Encapsulation**: Private/protected/public access modifiers
- **Abstract classes**: `BaseEntity` with pure virtual functions

### Modern C++ Features
- **Smart Pointers**: `std::unique_ptr`, `std::shared_ptr`, `std::weak_ptr`
- **RAII**: Resource management through constructors/destructors
- **Move Semantics**: Efficient resource transfer
- **Lambda Expressions**: Anonymous functions with captures
- **Range-based for loops**: Modern iteration syntax
- **Auto type deduction**: Automatic type inference

### Template Programming
- **Class Templates**: `Repository<T>`, `StatisticsCollector<T>`
- **Function Templates**: Generic utility functions
- **Template Specialization**: Specialized `StatisticsCollector<std::string>`
- **SFINAE**: Template metaprogramming techniques

### STL Usage
- **Containers**: `std::vector`, `std::unordered_map`, `std::queue`
- **Algorithms**: `std::sort`, `std::find`, `std::transform`, `std::count_if`
- **Iterators**: Container traversal and manipulation
- **Function Objects**: `std::function`, predicates

### Design Patterns
- **Singleton**: `UserRepository` singleton instance
- **Factory**: `UserFactory` for creating different user types
- **Observer**: Event system with subject/observer pattern
- **Repository**: Data access abstraction layer

### Concurrency
- **Thread Pool**: Custom thread pool implementation
- **Futures/Promises**: Asynchronous task execution
- **Mutexes**: Thread synchronization
- **Condition Variables**: Thread coordination

### Exception Handling
- **Custom Exceptions**: `DatabaseError`, `ValidationError`
- **RAII Exception Safety**: Resource cleanup on exceptions
- **Exception Specifications**: `noexcept` usage

### Namespace Organization
- **Nested Namespaces**: `app::models`, `app::utils`
- **Anonymous Namespaces**: Internal linkage functions
- **Using Declarations**: Selective namespace importing

## Building the Application

### Using Make
```bash
# Build the application
make all

# Build debug version
make debug

# Run the application
make run

# Run with valgrind (memory checking)
make valgrind

# Clean build artifacts
make clean

# Show all available targets
make help
```

### Using CMake
```bash
# Create build directory
mkdir build && cd build

# Configure build
cmake ..

# Build the application
cmake --build .

# Run the application
./bin/cpp_test_app

# Run tests
cmake --build . --target test
```

### Build Requirements
- **Compiler**: GCC 7+ or Clang 5+ with C++17 support
- **CMake**: Version 3.16 or later (for CMake builds)
- **Make**: GNU Make (for Makefile builds)
- **Threading**: POSIX threads (pthread)

## Installation

### Dependencies (Ubuntu/Debian)
```bash
sudo apt-get update
sudo apt-get install build-essential g++ make cmake valgrind
```

### Optional Tools
```bash
# For code formatting
sudo apt-get install clang-format

# For static analysis
sudo apt-get install cppcheck

# For documentation generation
sudo apt-get install doxygen graphviz
```

## Running the Application

### Basic Execution
```bash
./cpp_test_app [config_file]
```

### With Custom Configuration
```bash
./cpp_test_app my_config.json
```

### Memory Analysis
```bash
valgrind --leak-check=full ./cpp_test_app
```

## Configuration

The application uses a JSON configuration file (`config.json`):

```json
{
  "database_url": "sqlite:///test.db",
  "log_level": "DEBUG",
  "max_users": "100",
  "enable_encryption": "true",
  "simulate_error": "false"
}
```

## Project Structure

```
cpp_app/
├── include/
│   ├── models.hpp      # Data models and business logic
│   └── utils.hpp       # Utility classes and functions
├── src/
│   └── main.cpp        # Main application implementation
├── build/              # Build artifacts (generated)
├── CMakeLists.txt      # CMake build configuration
├── Makefile           # Make build configuration
├── config.json.in     # Configuration template
└── README.md          # This file
```

## Code Organization

### Header Files
- **models.hpp**: Contains all data model classes, repositories, and business logic
- **utils.hpp**: Utility classes for logging, validation, string processing, etc.

### Source Files
- **main.cpp**: Main application entry point and application logic

### Key Classes
- `Application`: Main application controller
- `User` / `AdminUser`: User data models with inheritance
- `UserRepository`: Singleton data repository
- `UserManager`: Business logic layer
- `EventSystem`: Observer pattern implementation
- `ThreadPool`: Custom thread pool for concurrency
- `Logger`: Logging utility with multiple levels
- `Config`: Configuration management

## Development Features

### Code Quality
- **Compiler Warnings**: Extensive warning flags enabled
- **Static Analysis**: cppcheck integration
- **Memory Checking**: Valgrind integration
- **Code Formatting**: clang-format support

### Build Targets
- `all` / `debug`: Build release/debug versions
- `run` / `test`: Execute the application
- `clean`: Remove build artifacts
- `format`: Format code with clang-format
- `analyze`: Run static analysis
- `valgrind`: Memory leak detection
- `info`: Display build information

## Testing with EMBARGO

This application is specifically designed to test the EMBARGO framework's C++ parsing capabilities. It includes:

1. **Complex Inheritance Hierarchies**: Multiple levels of class inheritance
2. **Template Usage**: Various template patterns and specializations
3. **Namespace Organization**: Nested namespaces and using declarations
4. **Modern C++ Constructs**: Smart pointers, lambdas, auto, etc.
5. **STL Integration**: Extensive use of standard library components
6. **Design Patterns**: Common software design patterns
7. **Cross-Module Dependencies**: Inter-class and inter-namespace relationships

Run EMBARGO on this directory to analyze the dependency graph and verify proper C++ parsing.

## License

This test application is part of the EMBARGO framework and follows the same MIT license.
