#pragma once

#include <string>
#include <vector>
#include <unordered_map>
#include <memory>
#include <chrono>
#include <iostream>
#include <fstream>
#include <regex>
#include <algorithm>
#include <functional>
#include <thread>
#include <mutex>
#include <queue>
#include <future>

namespace app {
namespace utils {

// Logging utilities
enum class LogLevel {
    DEBUG = 0,
    INFO = 1,
    WARNING = 2,
    ERROR = 3,
    CRITICAL = 4
};

class Logger {
private:
    std::string name_;
    LogLevel level_;
    std::ofstream log_file_;
    mutable std::mutex log_mutex_;

public:
    explicit Logger(const std::string& name, LogLevel level = LogLevel::INFO);
    ~Logger();
    
    void set_level(LogLevel level) { level_ = level; }
    void set_log_file(const std::string& filename);
    
    void log(LogLevel level, const std::string& message) const;
    void debug(const std::string& message) const;
    void info(const std::string& message) const;
    void warning(const std::string& message) const;
    void error(const std::string& message) const;
    void critical(const std::string& message) const;
    
    // Template method for formatted logging
    template<typename... Args>
    void log_formatted(LogLevel level, const std::string& format, Args&&... args) const;
};

// Cryptographic utilities
namespace crypto {
    std::string calculate_hash(const std::string& input, const std::string& algorithm = "sha256");
    std::string hash_password(const std::string& password, const std::string& salt = "");
    std::string generate_salt(size_t length = 16);
    bool verify_password(const std::string& password, const std::string& hash, const std::string& salt = "");
    
    // Template function for hashing different types
    template<typename T>
    std::string hash_object(const T& obj);
}

// Validation utilities
namespace validation {
    bool validate_email(const std::string& email);
    bool validate_username(const std::string& username);
    bool validate_password(const std::string& password);
    bool validate_url(const std::string& url);
    
    // Template validation function
    template<typename T, typename Validator>
    bool validate_with_predicate(const T& value, Validator validator);
    
    class ValidationRule {
    public:
        virtual ~ValidationRule() = default;
        virtual bool validate(const std::string& value) const = 0;
        virtual std::string get_error_message() const = 0;
    };
    
    class LengthRule : public ValidationRule {
    private:
        size_t min_length_;
        size_t max_length_;
        
    public:
        LengthRule(size_t min_len, size_t max_len) : min_length_(min_len), max_length_(max_len) {}
        bool validate(const std::string& value) const override;
        std::string get_error_message() const override;
    };
    
    class RegexRule : public ValidationRule {
    private:
        std::regex pattern_;
        std::string error_msg_;
        
    public:
        RegexRule(const std::string& pattern, const std::string& error_msg);
        bool validate(const std::string& value) const override;
        std::string get_error_message() const override;
    };
    
    class Validator {
    private:
        std::vector<std::unique_ptr<ValidationRule>> rules_;
        
    public:
        void add_rule(std::unique_ptr<ValidationRule> rule);
        bool validate(const std::string& value, std::vector<std::string>& errors) const;
    };
}

// String utilities
namespace string_utils {
    std::string to_lower(const std::string& str);
    std::string to_upper(const std::string& str);
    std::string trim(const std::string& str);
    std::string trim_left(const std::string& str);
    std::string trim_right(const std::string& str);
    
    std::vector<std::string> split(const std::string& str, const std::string& delimiter);
    std::string join(const std::vector<std::string>& strings, const std::string& delimiter);
    
    bool starts_with(const std::string& str, const std::string& prefix);
    bool ends_with(const std::string& str, const std::string& suffix);
    bool contains(const std::string& str, const std::string& substring);
    
    std::string replace_all(const std::string& str, const std::string& from, const std::string& to);
    
    // Template function for converting types to string
    template<typename T>
    std::string to_string(const T& value);
    
    // Template function for parsing strings to types
    template<typename T>
    T from_string(const std::string& str);
}

// File utilities
namespace file_utils {
    bool file_exists(const std::string& path);
    bool directory_exists(const std::string& path);
    std::string read_file(const std::string& path);
    bool write_file(const std::string& path, const std::string& content);
    
    std::vector<std::string> list_directory(const std::string& path);
    std::vector<std::string> find_files(const std::string& directory, const std::string& pattern);
    
    std::string get_file_extension(const std::string& path);
    std::string get_filename(const std::string& path);
    std::string get_directory(const std::string& path);
    
    size_t get_file_size(const std::string& path);
    std::chrono::system_clock::time_point get_last_modified(const std::string& path);
}

// Configuration management
class Config {
private:
    std::unordered_map<std::string, std::string> config_data_;
    std::string config_file_;

public:
    explicit Config(const std::string& config_file = "");
    
    bool load_from_file(const std::string& filename);
    bool save_to_file(const std::string& filename = "") const;
    
    void set(const std::string& key, const std::string& value);
    std::string get(const std::string& key, const std::string& default_value = "") const;
    
    template<typename T>
    void set_typed(const std::string& key, const T& value);
    
    template<typename T>
    T get_typed(const std::string& key, const T& default_value = T{}) const;
    
    bool has_key(const std::string& key) const;
    void remove(const std::string& key);
    void clear();
    
    std::vector<std::string> get_keys() const;
    std::unordered_map<std::string, std::string> get_all() const { return config_data_; }
};

// Thread pool for concurrent operations
class ThreadPool {
private:
    std::vector<std::thread> workers_;
    std::queue<std::function<void()>> tasks_;
    std::mutex queue_mutex_;
    std::condition_variable condition_;
    bool stop_;

public:
    explicit ThreadPool(size_t threads = std::thread::hardware_concurrency());
    ~ThreadPool();
    
    template<class F, class... Args>
    auto enqueue(F&& f, Args&&... args) -> std::future<typename std::result_of<F(Args...)>::type>;
    
    size_t size() const { return workers_.size(); }
    void wait_for_completion();
};

// JSON utilities (simplified)
namespace json {
    std::string escape_string(const std::string& str);
    std::string array_to_json(const std::vector<std::string>& arr);
    std::string map_to_json(const std::unordered_map<std::string, std::string>& map);
    
    template<typename T>
    std::string value_to_json(const T& value);
    
    class JsonBuilder {
    private:
        std::string json_str_;
        bool first_element_;
        
    public:
        JsonBuilder();
        
        JsonBuilder& start_object();
        JsonBuilder& end_object();
        JsonBuilder& start_array();
        JsonBuilder& end_array();
        
        JsonBuilder& add_key(const std::string& key);
        JsonBuilder& add_string(const std::string& value);
        JsonBuilder& add_number(double value);
        JsonBuilder& add_bool(bool value);
        JsonBuilder& add_null();
        
        template<typename T>
        JsonBuilder& add_value(const T& value);
        
        std::string build() const { return json_str_; }
    };
}

// Time utilities
namespace time_utils {
    std::string format_timestamp(const std::chrono::system_clock::time_point& time, 
                                const std::string& format = "%Y-%m-%d %H:%M:%S");
    std::chrono::system_clock::time_point parse_timestamp(const std::string& time_str, 
                                                         const std::string& format = "%Y-%m-%d %H:%M:%S");
    
    std::string format_duration(const std::chrono::milliseconds& duration);
    
    class Timer {
    private:
        std::chrono::high_resolution_clock::time_point start_time_;
        
    public:
        Timer();
        void reset();
        std::chrono::milliseconds elapsed() const;
        std::string elapsed_string() const;
    };
    
    class Stopwatch {
    private:
        std::chrono::high_resolution_clock::time_point start_time_;
        std::chrono::milliseconds total_elapsed_;
        bool running_;
        
    public:
        Stopwatch();
        
        void start();
        void stop();
        void reset();
        void restart();
        
        std::chrono::milliseconds elapsed() const;
        bool is_running() const { return running_; }
    };
}

// Memory utilities
namespace memory {
    template<typename T, typename... Args>
    std::unique_ptr<T> make_unique(Args&&... args) {
        return std::unique_ptr<T>(new T(std::forward<Args>(args)...));
    }
    
    template<typename T>
    class ObjectPool {
    private:
        std::queue<std::unique_ptr<T>> pool_;
        std::mutex pool_mutex_;
        std::function<std::unique_ptr<T>()> factory_;
        
    public:
        explicit ObjectPool(std::function<std::unique_ptr<T>()> factory);
        
        std::unique_ptr<T> acquire();
        void release(std::unique_ptr<T> obj);
        size_t size() const;
    };
}

} // namespace utils
} // namespace app
