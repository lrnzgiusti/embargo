# Makefile for C++ Test Application
# Demonstrates various C++ language constructs for the EMBARGO framework

# Compiler settings
CXX = g++
CXXFLAGS = -std=c++17 -Wall -Wextra -Wpedantic -O2
DEBUG_FLAGS = -std=c++17 -Wall -Wextra -Wpedantic -g -O0 -DDEBUG
INCLUDE_DIRS = -Iinclude
LDFLAGS = -pthread

# Directories
SRC_DIR = src
INCLUDE_DIR = include
BUILD_DIR = build
OBJ_DIR = $(BUILD_DIR)/obj
BIN_DIR = $(BUILD_DIR)/bin

# Source files
SOURCES = $(wildcard $(SRC_DIR)/*.cpp)
OBJECTS = $(SOURCES:$(SRC_DIR)/%.cpp=$(OBJ_DIR)/%.o)
HEADERS = $(wildcard $(INCLUDE_DIR)/*.hpp)

# Target executable
TARGET = $(BIN_DIR)/cpp_test_app
DEBUG_TARGET = $(BIN_DIR)/cpp_test_app_debug

# Default target
all: $(TARGET)

# Debug target
debug: CXXFLAGS = $(DEBUG_FLAGS)
debug: $(DEBUG_TARGET)

# Create build directories
$(OBJ_DIR):
	@mkdir -p $(OBJ_DIR)

$(BIN_DIR):
	@mkdir -p $(BIN_DIR)

# Compile object files
$(OBJ_DIR)/%.o: $(SRC_DIR)/%.cpp $(HEADERS) | $(OBJ_DIR)
	@echo "Compiling $<..."
	$(CXX) $(CXXFLAGS) $(INCLUDE_DIRS) -c $< -o $@

# Link executable
$(TARGET): $(OBJECTS) | $(BIN_DIR)
	@echo "Linking $@..."
	$(CXX) $(OBJECTS) $(LDFLAGS) -o $@
	@echo "Build complete: $@"

# Debug executable
$(DEBUG_TARGET): $(OBJECTS) | $(BIN_DIR)
	@echo "Linking debug version $@..."
	$(CXX) $(OBJECTS) $(LDFLAGS) -o $@
	@echo "Debug build complete: $@"

# Run the application
run: $(TARGET)
	@echo "Running C++ test application..."
	./$(TARGET)

# Run with debug
run-debug: $(DEBUG_TARGET)
	@echo "Running C++ test application (debug)..."
	./$(DEBUG_TARGET)

# Run with valgrind (memory checking)
valgrind: $(DEBUG_TARGET)
	@echo "Running with valgrind..."
	valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes ./$(DEBUG_TARGET)

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	rm -rf $(BUILD_DIR)

# Clean and rebuild
rebuild: clean all

# Install dependencies (for Ubuntu/Debian)
install-deps:
	@echo "Installing dependencies..."
	sudo apt-get update
	sudo apt-get install -y build-essential g++ make cmake valgrind

# Format code (requires clang-format)
format:
	@echo "Formatting code..."
	find $(SRC_DIR) $(INCLUDE_DIR) -name "*.cpp" -o -name "*.hpp" | xargs clang-format -i

# Static analysis (requires cppcheck)
analyze:
	@echo "Running static analysis..."
	cppcheck --enable=all --inconclusive --std=c++17 $(SRC_DIR) $(INCLUDE_DIR)

# Check for memory leaks in a quick way
quick-test: $(TARGET)
	@echo "Running quick functionality test..."
	timeout 10s ./$(TARGET) || true

# Print build information
info:
	@echo "=== Build Information ==="
	@echo "Compiler: $(CXX)"
	@echo "Flags: $(CXXFLAGS)"
	@echo "Include dirs: $(INCLUDE_DIRS)"
	@echo "Link flags: $(LDFLAGS)"
	@echo "Sources: $(SOURCES)"
	@echo "Objects: $(OBJECTS)"
	@echo "Target: $(TARGET)"
	@echo "========================="

# Create a simple config file for testing
config:
	@echo "Creating default config.json..."
	@echo '{' > config.json
	@echo '  "database_url": "sqlite:///test.db",' >> config.json
	@echo '  "log_level": "DEBUG",' >> config.json
	@echo '  "max_users": "100",' >> config.json
	@echo '  "enable_encryption": "true",' >> config.json
	@echo '  "simulate_error": "false"' >> config.json
	@echo '}' >> config.json
	@echo "Config file created: config.json"

# Help target
help:
	@echo "Available targets:"
	@echo "  all          - Build the application (default)"
	@echo "  debug        - Build debug version with symbols"
	@echo "  run          - Build and run the application"
	@echo "  run-debug    - Build and run debug version"
	@echo "  valgrind     - Run with valgrind memory checking"
	@echo "  clean        - Remove build artifacts"
	@echo "  rebuild      - Clean and rebuild"
	@echo "  install-deps - Install required dependencies"
	@echo "  format       - Format code with clang-format"
	@echo "  analyze      - Run static analysis with cppcheck"
	@echo "  quick-test   - Run a quick functionality test"
	@echo "  config       - Create a default config.json file"
	@echo "  info         - Display build information"
	@echo "  help         - Show this help message"

.PHONY: all debug run run-debug valgrind clean rebuild install-deps format analyze quick-test info config help
