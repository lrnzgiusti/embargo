#pragma once

#include <string>
#include <vector>
#include <memory>
#include <unordered_map>
#include <chrono>
#include <iostream>
#include <exception>

namespace app {
namespace models {

// Custom exception classes
class DatabaseError : public std::exception {
private:
    std::string message_;

public:
    explicit DatabaseError(const std::string& message) : message_(message) {}
    
    const char* what() const noexcept override {
        return message_.c_str();
    }
};

class ValidationError : public std::exception {
private:
    std::string message_;

public:
    explicit ValidationError(const std::string& message) : message_(message) {}
    
    const char* what() const noexcept override {
        return message_.c_str();
    }
};

// Abstract base entity class
class BaseEntity {
protected:
    std::chrono::system_clock::time_point created_at_;
    std::chrono::system_clock::time_point updated_at_;

public:
    BaseEntity();
    virtual ~BaseEntity() = default;
    
    // Pure virtual methods
    virtual std::unordered_map<std::string, std::string> to_dict() const = 0;
    virtual std::string to_json() const = 0;
    
    // Common methods
    void update_timestamp();
    std::chrono::system_clock::time_point get_created_at() const;
    std::chrono::system_clock::time_point get_updated_at() const;
};

// User class with inheritance
class User : public BaseEntity {
private:
    std::string username_;
    std::string email_;
    std::string password_hash_;
    bool is_active_;

public:
    User(const std::string& username, const std::string& email, const std::string& password = "");
    virtual ~User() = default;
    
    // Getters
    const std::string& get_username() const { return username_; }
    const std::string& get_email() const { return email_; }
    bool is_active() const { return is_active_; }
    
    // Setters
    void set_email(const std::string& email);
    void set_active(bool active) { is_active_ = active; }
    void set_password(const std::string& password);
    
    // Virtual methods
    virtual std::unordered_map<std::string, std::string> to_dict() const override;
    virtual std::string to_json() const override;
    virtual std::string get_role() const { return "user"; }
    
    // Operator overloading
    bool operator==(const User& other) const;
    friend std::ostream& operator<<(std::ostream& os, const User& user);
};

// AdminUser inheriting from User
class AdminUser : public User {
private:
    std::vector<std::string> permissions_;
    int access_level_;

public:
    AdminUser(const std::string& username, const std::string& email, 
              const std::vector<std::string>& permissions, int access_level = 5);
    
    // Override virtual methods
    std::string get_role() const override { return "admin"; }
    std::unordered_map<std::string, std::string> to_dict() const override;
    std::string to_json() const override;
    
    // Admin-specific methods
    void add_permission(const std::string& permission);
    void remove_permission(const std::string& permission);
    const std::vector<std::string>& get_permissions() const { return permissions_; }
    int get_access_level() const { return access_level_; }
    void set_access_level(int level) { access_level_ = level; }
};

// Template class for generic repository pattern
template<typename T>
class Repository {
private:
    std::vector<std::unique_ptr<T>> items_;
    std::unordered_map<std::string, size_t> index_map_;

public:
    Repository() = default;
    virtual ~Repository() = default;
    
    // Non-copyable but movable
    Repository(const Repository&) = delete;
    Repository& operator=(const Repository&) = delete;
    Repository(Repository&&) = default;
    Repository& operator=(Repository&&) = default;
    
    void add(std::unique_ptr<T> item);
    T* find_by_id(const std::string& id) const;
    std::vector<T*> get_all() const;
    bool remove_by_id(const std::string& id);
    size_t size() const { return items_.size(); }
    
    // Template method for filtering
    template<typename Predicate>
    std::vector<T*> filter(Predicate pred) const;
};

// Specialized repository for Users
class UserRepository : public Repository<User> {
private:
    static std::unique_ptr<UserRepository> instance_;

public:
    UserRepository() = default;
    
    // Singleton pattern
    static UserRepository& get_instance();
    
    // User-specific methods
    User* find_by_username(const std::string& username) const;
    User* find_by_email(const std::string& email) const;
    std::vector<User*> find_active_users() const;
    std::vector<AdminUser*> find_admin_users() const;
    
    void create_user(const std::string& username, const std::string& email, const std::string& password = "");
    void create_admin_user(const std::string& username, const std::string& email, 
                          const std::vector<std::string>& permissions, int access_level = 5);
};

// UserManager class for business logic
class UserManager {
private:
    UserRepository& repository_;
    std::unordered_map<std::string, std::string> config_;

public:
    explicit UserManager(UserRepository& repo);
    
    void configure(const std::unordered_map<std::string, std::string>& config);
    
    // User management operations
    User* create_user(const std::string& username, const std::string& email, const std::string& password = "");
    AdminUser* create_admin_user(const std::string& username, const std::string& email, 
                                const std::vector<std::string>& permissions);
    bool delete_user(const std::string& username);
    User* authenticate_user(const std::string& username, const std::string& password);
    
    // Query methods
    std::vector<User*> get_all_users() const;
    std::vector<User*> search_users(const std::string& query) const;
    void print_user_statistics() const;
};

} // namespace models
} // namespace app
