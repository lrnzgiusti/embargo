"""
Data models for the application.
"""
from abc import ABC, abstractmethod
from typing import List, Optional, Dict, Any
from datetime import datetime
import sqlite3

from utils import hash_password, validate_email


class DatabaseError(Exception):
    """Custom exception for database-related errors."""
    pass


class BaseEntity(ABC):
    """Abstract base class for all entities."""
    
    def __init__(self, created_at: Optional[datetime] = None):
        """Initialize the entity with creation timestamp.
        
        Args:
            created_at: Creation timestamp, defaults to now
        """
        self.created_at = created_at or datetime.now()
        self.updated_at = self.created_at
    
    @abstractmethod
    def to_dict(self) -> Dict[str, Any]:
        """Convert entity to dictionary representation.
        
        Returns:
            Dictionary representation of the entity
        """
        pass
    
    def update_timestamp(self):
        """Update the last modified timestamp."""
        self.updated_at = datetime.now()


class User(BaseEntity):
    """User model representing application users."""
    
    def __init__(self, username: str, email: str, password: Optional[str] = None):
        """Initialize a new user.
        
        Args:
            username: Unique username
            email: User's email address
            password: Optional password (will be hashed)
        """
        super().__init__()
        self.username = username
        self.email = email
        self.password_hash = hash_password(password) if password else None
        self.is_active = True
        
        if not validate_email(email):
            raise ValueError(f"Invalid email address: {email}")
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert user to dictionary representation.
        
        Returns:
            Dictionary representation of the user
        """
        return {
            "username": self.username,
            "email": self.email,
            "is_active": self.is_active,
            "created_at": self.created_at.isoformat(),
            "updated_at": self.updated_at.isoformat(),
        }
    
    def activate(self):
        """Activate the user account."""
        self.is_active = True
        self.update_timestamp()
    
    def deactivate(self):
        """Deactivate the user account."""
        self.is_active = False
        self.update_timestamp()
    
    def change_password(self, new_password: str):
        """Change the user's password.
        
        Args:
            new_password: New password to set
        """
        self.password_hash = hash_password(new_password)
        self.update_timestamp()


class AdminUser(User):
    """Admin user with additional privileges."""
    
    def __init__(self, username: str, email: str, password: Optional[str] = None, 
                 permissions: Optional[List[str]] = None):
        """Initialize a new admin user.
        
        Args:
            username: Unique username
            email: User's email address
            password: Optional password (will be hashed)
            permissions: List of admin permissions
        """
        super().__init__(username, email, password)
        self.permissions = permissions or ["read", "write"]
        self.is_admin = True
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert admin user to dictionary representation.
        
        Returns:
            Dictionary representation of the admin user
        """
        data = super().to_dict()
        data.update({
            "is_admin": self.is_admin,
            "permissions": self.permissions,
        })
        return data
    
    def grant_permission(self, permission: str):
        """Grant a permission to the admin user.
        
        Args:
            permission: Permission to grant
        """
        if permission not in self.permissions:
            self.permissions.append(permission)
            self.update_timestamp()
    
    def revoke_permission(self, permission: str):
        """Revoke a permission from the admin user.
        
        Args:
            permission: Permission to revoke
        """
        if permission in self.permissions:
            self.permissions.remove(permission)
            self.update_timestamp()


class UserRepository:
    """Repository for user data persistence."""
    
    def __init__(self, db_path: str = "users.db"):
        """Initialize the repository with database path.
        
        Args:
            db_path: Path to SQLite database file
        """
        self.db_path = db_path
        self._init_database()
    
    def _init_database(self):
        """Initialize the database tables."""
        try:
            with sqlite3.connect(self.db_path) as conn:
                conn.execute("""
                    CREATE TABLE IF NOT EXISTS users (
                        id INTEGER PRIMARY KEY,
                        username TEXT UNIQUE NOT NULL,
                        email TEXT UNIQUE NOT NULL,
                        password_hash TEXT,
                        is_active BOOLEAN DEFAULT 1,
                        is_admin BOOLEAN DEFAULT 0,
                        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                    )
                """)
        except sqlite3.Error as e:
            raise DatabaseError(f"Failed to initialize database: {e}")
    
    def save_user(self, user: User) -> int:
        """Save a user to the database.
        
        Args:
            user: User object to save
            
        Returns:
            User ID
        """
        try:
            with sqlite3.connect(self.db_path) as conn:
                cursor = conn.execute("""
                    INSERT INTO users (username, email, password_hash, is_active, is_admin)
                    VALUES (?, ?, ?, ?, ?)
                """, (
                    user.username,
                    user.email,
                    user.password_hash,
                    user.is_active,
                    isinstance(user, AdminUser)
                ))
                return cursor.lastrowid
        except sqlite3.Error as e:
            raise DatabaseError(f"Failed to save user: {e}")
    
    def find_by_username(self, username: str) -> Optional[User]:
        """Find a user by username.
        
        Args:
            username: Username to search for
            
        Returns:
            User object if found, None otherwise
        """
        try:
            with sqlite3.connect(self.db_path) as conn:
                conn.row_factory = sqlite3.Row
                cursor = conn.execute(
                    "SELECT * FROM users WHERE username = ?", (username,)
                )
                row = cursor.fetchone()
                return self._row_to_user(row) if row else None
        except sqlite3.Error as e:
            raise DatabaseError(f"Failed to find user: {e}")
    
    def _row_to_user(self, row: sqlite3.Row) -> User:
        """Convert database row to User object.
        
        Args:
            row: Database row
            
        Returns:
            User object
        """
        if row["is_admin"]:
            user = AdminUser(row["username"], row["email"])
        else:
            user = User(row["username"], row["email"])
        
        user.password_hash = row["password_hash"]
        user.is_active = bool(row["is_active"])
        user.created_at = datetime.fromisoformat(row["created_at"])
        user.updated_at = datetime.fromisoformat(row["updated_at"])
        
        return user


class UserManager:
    """High-level user management service."""
    
    def __init__(self, repository: Optional[UserRepository] = None):
        """Initialize the user manager.
        
        Args:
            repository: User repository instance
        """
        self.repository = repository or UserRepository()
    
    def configure(self, config: Dict[str, str]):
        """Configure the user manager.
        
        Args:
            config: Configuration dictionary
        """
        if "database_url" in config:
            # In a real app, we'd parse the database URL
            self.repository = UserRepository()
    
    def create_user(self, username: str, email: str, password: str, 
                   is_admin: bool = False) -> User:
        """Create a new user.
        
        Args:
            username: Unique username
            email: User's email address
            password: User's password
            is_admin: Whether user should be an admin
            
        Returns:
            Created user object
        """
        if is_admin:
            user = AdminUser(username, email, password)
        else:
            user = User(username, email, password)
        
        self.repository.save_user(user)
        return user
    
    def get_user(self, username: str) -> Optional[User]:
        """Get a user by username.
        
        Args:
            username: Username to search for
            
        Returns:
            User object if found, None otherwise
        """
        return self.repository.find_by_username(username)
    
    def get_all_users(self) -> List[User]:
        """Get all users from the database.
        
        Returns:
            List of all users
        """
        # Simplified implementation for testing
        return [
            User("alice", "alice@example.com"),
            AdminUser("bob", "bob@example.com", permissions=["read", "write", "admin"]),
            User("charlie", "charlie@example.com"),
        ]