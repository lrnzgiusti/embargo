/**
 * Main application demonstrating various C++ language constructs.
 * This application showcases:
 * - Object-oriented programming (inheritance, polymorphism, encapsulation)
 * - Template programming and STL usage
 * - Modern C++ features (smart pointers, RAII, move semantics)
 * - Exception handling
 * - Namespace usage
 * - Design patterns (Singleton, Factory, Observer)
 */

#include <iostream>
#include <memory>
#include <vector>
#include <string>
#include <unordered_map>
#include <functional>
#include <algorithm>
#include <chrono>
#include <thread>
#include <future>
#include <exception>

#include "../include/models.hpp"
#include "../include/utils.hpp"

using namespace app::models;
using namespace app::utils;

// Forward declarations
class Application;
class EventSystem;

// Observer pattern implementation
class Observer {
public:
    virtual ~Observer() = default;
    virtual void notify(const std::string& event, const std::unordered_map<std::string, std::string>& data) = 0;
};

class Subject {
private:
    std::vector<std::weak_ptr<Observer>> observers_;

public:
    void attach(std::shared_ptr<Observer> observer) {
        observers_.push_back(observer);
    }
    
    void detach(std::shared_ptr<Observer> observer) {
        observers_.erase(
            std::remove_if(observers_.begin(), observers_.end(),
                [&](const std::weak_ptr<Observer>& weak_obs) {
                    return weak_obs.expired() || weak_obs.lock() == observer;
                }),
            observers_.end()
        );
    }
    
    void notify_observers(const std::string& event, const std::unordered_map<std::string, std::string>& data = {}) {
        for (auto it = observers_.begin(); it != observers_.end();) {
            if (auto observer = it->lock()) {
                observer->notify(event, data);
                ++it;
            } else {
                it = observers_.erase(it);
            }
        }
    }
};

// Event system for application events
class EventSystem : public Observer {
private:
    Logger logger_;

public:
    EventSystem() : logger_("EventSystem") {}

    void notify(const std::string& event, const std::unordered_map<std::string, std::string>& data) override {
        logger_.info("Event received: " + event);
        
        for (const auto& [key, value] : data) {
            logger_.debug("  " + key + ": " + value);
        }
        
        // Process different event types
        if (event == "user_created") {
            handle_user_created(data);
        } else if (event == "user_deleted") {
            handle_user_deleted(data);
        } else if (event == "application_started") {
            handle_application_started(data);
        } else if (event == "application_stopped") {
            handle_application_stopped(data);
        }
    }

private:
    void handle_user_created(const std::unordered_map<std::string, std::string>& data) {
        auto username_it = data.find("username");
        if (username_it != data.end()) {
            logger_.info("New user created: " + username_it->second);
        }
    }
    
    void handle_user_deleted(const std::unordered_map<std::string, std::string>& data) {
        auto username_it = data.find("username");
        if (username_it != data.end()) {
            logger_.warning("User deleted: " + username_it->second);
        }
    }
    
    void handle_application_started(const std::unordered_map<std::string, std::string>& data) {
        logger_.info("Application has started successfully");
    }
    
    void handle_application_stopped(const std::unordered_map<std::string, std::string>& data) {
        logger_.info("Application is shutting down");
    }
};

// Statistics collector using template specialization
template<typename T>
class StatisticsCollector {
private:
    std::vector<T> data_;

public:
    void add(const T& value) {
        data_.push_back(value);
    }
    
    size_t count() const {
        return data_.size();
    }
    
    T sum() const {
        T total{};
        for (const auto& value : data_) {
            total += value;
        }
        return total;
    }
    
    double average() const {
        if (data_.empty()) return 0.0;
        return static_cast<double>(sum()) / data_.size();
    }
    
    T min() const {
        if (data_.empty()) return T{};
        return *std::min_element(data_.begin(), data_.end());
    }
    
    T max() const {
        if (data_.empty()) return T{};
        return *std::max_element(data_.begin(), data_.end());
    }
    
    void clear() {
        data_.clear();
    }
};

// Specialized template for string statistics
template<>
class StatisticsCollector<std::string> {
private:
    std::vector<std::string> data_;

public:
    void add(const std::string& value) {
        data_.push_back(value);
    }
    
    size_t count() const {
        return data_.size();
    }
    
    double average_length() const {
        if (data_.empty()) return 0.0;
        
        size_t total_length = 0;
        for (const auto& str : data_) {
            total_length += str.length();
        }
        return static_cast<double>(total_length) / data_.size();
    }
    
    std::string longest() const {
        if (data_.empty()) return "";
        
        return *std::max_element(data_.begin(), data_.end(),
            [](const std::string& a, const std::string& b) {
                return a.length() < b.length();
            });
    }
    
    std::string shortest() const {
        if (data_.empty()) return "";
        
        return *std::min_element(data_.begin(), data_.end(),
            [](const std::string& a, const std::string& b) {
                return a.length() < b.length();
            });
    }
    
    void clear() {
        data_.clear();
    }
};

// Factory pattern for creating users
class UserFactory {
public:
    enum class UserType {
        REGULAR,
        ADMIN,
        GUEST
    };
    
    static std::unique_ptr<User> create_user(UserType type, const std::string& username, 
                                           const std::string& email, 
                                           const std::vector<std::string>& permissions = {}) {
        switch (type) {
            case UserType::REGULAR:
                return std::make_unique<User>(username, email);
            
            case UserType::ADMIN:
                return std::make_unique<AdminUser>(username, email, permissions, 5);
            
            case UserType::GUEST:
                return std::make_unique<User>("guest_" + username, email);
            
            default:
                throw std::invalid_argument("Unknown user type");
        }
    }
    
    static std::vector<std::unique_ptr<User>> create_sample_users() {
        std::vector<std::unique_ptr<User>> users;
        
        // Create regular users
        users.push_back(create_user(UserType::REGULAR, "alice", "alice@example.com"));
        users.push_back(create_user(UserType::REGULAR, "bob", "bob@example.com"));
        users.push_back(create_user(UserType::REGULAR, "charlie", "charlie@example.com"));
        
        // Create admin users
        std::vector<std::string> admin_permissions = {"read", "write", "delete", "admin"};
        users.push_back(create_user(UserType::ADMIN, "admin", "admin@example.com", admin_permissions));
        
        std::vector<std::string> moderator_permissions = {"read", "write", "moderate"};
        users.push_back(create_user(UserType::ADMIN, "moderator", "mod@example.com", moderator_permissions));
        
        // Create guest users
        users.push_back(create_user(UserType::GUEST, "visitor1", "visitor1@example.com"));
        users.push_back(create_user(UserType::GUEST, "visitor2", "visitor2@example.com"));
        
        return users;
    }
};

// Main application class demonstrating composition and dependency injection
class Application : public Subject {
private:
    Logger logger_;
    Config config_;
    UserManager user_manager_;
    std::shared_ptr<EventSystem> event_system_;
    StatisticsCollector<int> operation_times_;
    StatisticsCollector<std::string> processed_usernames_;
    time_utils::Timer app_timer_;
    std::unique_ptr<ThreadPool> thread_pool_;

public:
    Application(const std::string& config_file = "config.json") 
        : logger_("Application"), 
          config_(config_file),
          user_manager_(UserRepository::get_instance()),
          event_system_(std::make_shared<EventSystem>()),
          thread_pool_(std::make_unique<ThreadPool>(4)) {
        
        // Attach event system as observer
        attach(event_system_);
        
        logger_.info("Application initialized");
    }
    
    ~Application() {
        notify_observers("application_stopped");
        logger_.info("Application destroyed");
    }
    
    int run() {
        try {
            logger_.info("Starting application");
            notify_observers("application_started");
            
            initialize_system();
            demonstrate_features();
            process_users();
            run_concurrent_operations();
            display_statistics();
            
            logger_.info("Application completed successfully");
            return 0;
            
        } catch (const DatabaseError& e) {
            logger_.error("Database error: " + std::string(e.what()));
            return 2;
        } catch (const ValidationError& e) {
            logger_.error("Validation error: " + std::string(e.what()));
            return 3;
        } catch (const std::exception& e) {
            logger_.error("Unexpected error: " + std::string(e.what()));
            return 1;
        }
    }

private:
    void initialize_system() {
        logger_.info("Initializing system components");
        
        // Load configuration
        if (!config_.load_from_file("config.json")) {
            logger_.warning("Could not load config file, using defaults");
            setup_default_config();
        }
        
        // Configure user manager
        user_manager_.configure(config_.get_all());
        
        logger_.info("System initialized successfully");
    }
    
    void setup_default_config() {
        config_.set("database_url", "sqlite:///users.db");
        config_.set("log_level", "INFO");
        config_.set("max_users", "1000");
        config_.set("enable_encryption", "true");
        config_.save_to_file("config.json");
    }
    
    void demonstrate_features() {
        logger_.info("Demonstrating C++ features");
        
        // Demonstrate template usage
        demonstrate_templates();
        
        // Demonstrate STL algorithms
        demonstrate_stl_algorithms();
        
        // Demonstrate lambda expressions
        demonstrate_lambdas();
        
        // Demonstrate exception handling
        demonstrate_exception_handling();
    }
    
    void demonstrate_templates() {
        logger_.debug("Demonstrating template usage");
        
        // Use template repository
        Repository<std::string> string_repo;
        auto str_ptr = std::make_unique<std::string>("test_string");
        string_repo.add(std::move(str_ptr));
        
        // Use template statistics collector
        StatisticsCollector<int> int_stats;
        int_stats.add(10);
        int_stats.add(20);
        int_stats.add(30);
        
        logger_.debug("Integer stats - Count: " + std::to_string(int_stats.count()) +
                     ", Average: " + std::to_string(int_stats.average()));
        
        StatisticsCollector<std::string> string_stats;
        string_stats.add("hello");
        string_stats.add("world");
        string_stats.add("cpp");
        
        logger_.debug("String stats - Count: " + std::to_string(string_stats.count()) +
                     ", Average length: " + std::to_string(string_stats.average_length()));
    }
    
    void demonstrate_stl_algorithms() {
        logger_.debug("Demonstrating STL algorithms");
        
        std::vector<int> numbers = {5, 2, 8, 1, 9, 3};
        
        // Sort
        std::sort(numbers.begin(), numbers.end());
        
        // Find
        auto it = std::find(numbers.begin(), numbers.end(), 8);
        if (it != numbers.end()) {
            logger_.debug("Found 8 at position: " + std::to_string(std::distance(numbers.begin(), it)));
        }
        
        // Transform
        std::vector<int> squared;
        std::transform(numbers.begin(), numbers.end(), std::back_inserter(squared),
                      [](int n) { return n * n; });
        
        // Count_if
        auto even_count = std::count_if(numbers.begin(), numbers.end(),
                                       [](int n) { return n % 2 == 0; });
        logger_.debug("Even numbers count: " + std::to_string(even_count));
    }
    
    void demonstrate_lambdas() {
        logger_.debug("Demonstrating lambda expressions");
        
        // Simple lambda
        auto add = [](int a, int b) { return a + b; };
        int result = add(5, 3);
        logger_.debug("Lambda add result: " + std::to_string(result));
        
        // Lambda with capture
        int multiplier = 10;
        auto multiply_by_factor = [multiplier](int value) { return value * multiplier; };
        int multiplied = multiply_by_factor(5);
        logger_.debug("Lambda multiply result: " + std::to_string(multiplied));
        
        // Lambda as function parameter
        auto process_numbers = [this](const std::vector<int>& nums, std::function<void(int)> processor) {
            for (int num : nums) {
                processor(num);
            }
        };
        
        std::vector<int> test_nums = {1, 2, 3, 4, 5};
        process_numbers(test_nums, [this](int n) {
            operation_times_.add(n);
            logger_.debug("Processed number: " + std::to_string(n));
        });
    }
    
    void demonstrate_exception_handling() {
        logger_.debug("Demonstrating exception handling");
        
        try {
            // Simulate database error
            if (config_.get("simulate_error") == "true") {
                throw DatabaseError("Simulated database connection failure");
            }
            
            // Simulate validation error
            validation::Validator validator;
            validator.add_rule(std::make_unique<validation::LengthRule>(3, 20));
            validator.add_rule(std::make_unique<validation::RegexRule>("^[a-zA-Z0-9_]+$", "Invalid characters"));
            
            std::vector<std::string> errors;
            if (!validator.validate("ab", errors)) {
                throw ValidationError("Username validation failed: " + errors[0]);
            }
            
        } catch (const DatabaseError& e) {
            logger_.warning("Caught database error (expected): " + std::string(e.what()));
        } catch (const ValidationError& e) {
            logger_.warning("Caught validation error (expected): " + std::string(e.what()));
        }
    }
    
    void process_users() {
        logger_.info("Processing users");
        
        // Create sample users using factory pattern
        auto users = UserFactory::create_sample_users();
        
        // Process each user
        for (auto& user : users) {
            try {
                process_single_user(*user);
                processed_usernames_.add(user->get_username());
                
                // Notify observers about user creation
                std::unordered_map<std::string, std::string> event_data = {
                    {"username", user->get_username()},
                    {"email", user->get_email()},
                    {"role", user->get_role()}
                };
                notify_observers("user_created", event_data);
                
            } catch (const std::exception& e) {
                logger_.error("Failed to process user " + user->get_username() + ": " + e.what());
            }
        }
        
        // Demonstrate polymorphism
        demonstrate_polymorphism(users);
    }
    
    void process_single_user(const User& user) {
        auto start_time = std::chrono::high_resolution_clock::now();
        
        // Simulate processing time
        std::this_thread::sleep_for(std::chrono::milliseconds(10));
        
        // Process user data
        auto user_data = user.to_dict();
        std::string json_data = user.to_json();
        
        // Calculate hash for user data
        std::string user_hash = crypto::calculate_hash(user.get_username() + user.get_email());
        
        logger_.debug("Processed user: " + user.get_username() + " (hash: " + user_hash.substr(0, 8) + "...)");
        
        auto end_time = std::chrono::high_resolution_clock::now();
        auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end_time - start_time);
        operation_times_.add(static_cast<int>(duration.count()));
    }
    
    void demonstrate_polymorphism(const std::vector<std::unique_ptr<User>>& users) {
        logger_.debug("Demonstrating polymorphism");
        
        for (const auto& user : users) {
            // Virtual function call
            std::string role = user->get_role();
            logger_.debug("User " + user->get_username() + " has role: " + role);
            
            // Dynamic casting
            if (auto admin_user = dynamic_cast<AdminUser*>(user.get())) {
                const auto& permissions = admin_user->get_permissions();
                std::string perms_str = string_utils::join(permissions, ", ");
                logger_.debug("Admin permissions: " + perms_str);
            }
        }
    }
    
    void run_concurrent_operations() {
        logger_.info("Running concurrent operations");
        
        std::vector<std::future<std::string>> futures;
        
        // Submit tasks to thread pool
        for (int i = 0; i < 10; ++i) {
            auto future = thread_pool_->enqueue([i, this]() -> std::string {
                // Simulate some work
                std::this_thread::sleep_for(std::chrono::milliseconds(50 + (i * 10)));
                
                std::string result = "Task " + std::to_string(i) + " completed";
                logger_.debug(result);
                return result;
            });
            
            futures.push_back(std::move(future));
        }
        
        // Wait for all tasks to complete
        for (auto& future : futures) {
            try {
                std::string result = future.get();
                // Results are already logged in the task
            } catch (const std::exception& e) {
                logger_.error("Concurrent task failed: " + std::string(e.what()));
            }
        }
        
        logger_.info("All concurrent operations completed");
    }
    
    void display_statistics() {
        logger_.info("Displaying application statistics");
        
        auto elapsed = app_timer_.elapsed();
        logger_.info("Total execution time: " + time_utils::format_duration(elapsed));
        
        logger_.info("Operation statistics:");
        logger_.info("  - Operations count: " + std::to_string(operation_times_.count()));
        logger_.info("  - Average operation time: " + std::to_string(operation_times_.average()) + "ms");
        logger_.info("  - Min operation time: " + std::to_string(operation_times_.min()) + "ms");
        logger_.info("  - Max operation time: " + std::to_string(operation_times_.max()) + "ms");
        
        logger_.info("Username statistics:");
        logger_.info("  - Processed usernames: " + std::to_string(processed_usernames_.count()));
        logger_.info("  - Average username length: " + std::to_string(processed_usernames_.average_length()));
        logger_.info("  - Longest username: " + processed_usernames_.longest());
        logger_.info("  - Shortest username: " + processed_usernames_.shortest());
    }
};

// Global utility functions
namespace {
    void print_application_info() {
        std::cout << "=== C++ Test Application ===" << std::endl;
        std::cout << "This application demonstrates various C++ language features:" << std::endl;
        std::cout << "- Object-oriented programming (inheritance, polymorphism)" << std::endl;
        std::cout << "- Template programming and STL usage" << std::endl;
        std::cout << "- Modern C++ features (smart pointers, RAII, move semantics)" << std::endl;
        std::cout << "- Design patterns (Singleton, Factory, Observer)" << std::endl;
        std::cout << "- Exception handling" << std::endl;
        std::cout << "- Concurrent programming" << std::endl;
        std::cout << "- Namespace usage" << std::endl;
        std::cout << "==============================" << std::endl;
    }
    
    void setup_signal_handlers() {
        // In a real application, you would set up signal handlers here
        // For demonstration purposes, we'll just log this action
        std::cout << "Signal handlers would be set up here" << std::endl;
    }
    
    bool validate_environment() {
        // Simulate environment validation
        return true;
    }
}

// Main entry point
int main(int argc, char* argv[]) {
    try {
        print_application_info();
        
        if (!validate_environment()) {
            std::cerr << "Environment validation failed" << std::endl;
            return 1;
        }
        
        setup_signal_handlers();
        
        // Parse command line arguments
        std::string config_file = "config.json";
        if (argc > 1) {
            config_file = argv[1];
        }
        
        // Create and run application
        Application app(config_file);
        int exit_code = app.run();
        
        std::cout << "Application finished with exit code: " << exit_code << std::endl;
        return exit_code;
        
    } catch (const std::exception& e) {
        std::cerr << "Unhandled exception: " << e.what() << std::endl;
        return 1;
    } catch (...) {
        std::cerr << "Unknown exception occurred" << std::endl;
        return 1;
    }
}
