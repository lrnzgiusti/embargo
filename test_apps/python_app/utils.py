"""
Utility functions and classes for the application.
"""
import hashlib
import re
import json
import logging
from typing import Any, Dict, Optional
from datetime import datetime


# Email validation regex pattern
EMAIL_PATTERN = re.compile(r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$')


def validate_email(email: str) -> bool:
    """Validate email address format.
    
    Args:
        email: Email address to validate
        
    Returns:
        True if email is valid, False otherwise
    """
    if not email or not isinstance(email, str):
        return False
    return bool(EMAIL_PATTERN.match(email))


def hash_password(password: str) -> str:
    """Hash a password using SHA-256.
    
    Args:
        password: Plain text password
        
    Returns:
        Hashed password
    """
    if not password:
        return ""
    
    salt = "embargo_salt_2023"  # In production, use random salt
    salted_password = f"{salt}{password}"
    return hashlib.sha256(salted_password.encode()).hexdigest()


def calculate_hash(data: str) -> str:
    """Calculate MD5 hash of given data.
    
    Args:
        data: Data to hash
        
    Returns:
        MD5 hash as hexadecimal string
    """
    return hashlib.md5(data.encode()).hexdigest()


def format_output(data: Dict[str, Any]) -> str:
    """Format data as pretty JSON string.
    
    Args:
        data: Data to format
        
    Returns:
        Formatted JSON string
    """
    return json.dumps(data, indent=2, default=str)


def timestamp_to_string(timestamp: datetime) -> str:
    """Convert timestamp to formatted string.
    
    Args:
        timestamp: Datetime object to format
        
    Returns:
        Formatted timestamp string
    """
    return timestamp.strftime("%Y-%m-%d %H:%M:%S")


class Logger:
    """Simple logging utility class."""
    
    def __init__(self, name: str, level: str = "INFO"):
        """Initialize logger with name and level.
        
        Args:
            name: Logger name
            level: Logging level (DEBUG, INFO, WARNING, ERROR)
        """
        self.name = name
        self.level = getattr(logging, level.upper(), logging.INFO)
        self.logger = logging.getLogger(name)
        self.logger.setLevel(self.level)
        
        if not self.logger.handlers:
            handler = logging.StreamHandler()
            formatter = logging.Formatter(
                '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
            )
            handler.setFormatter(formatter)
            self.logger.addHandler(handler)
    
    def debug(self, message: str):
        """Log debug message.
        
        Args:
            message: Message to log
        """
        self.logger.debug(message)
    
    def info(self, message: str):
        """Log info message.
        
        Args:
            message: Message to log
        """
        self.logger.info(message)
    
    def warning(self, message: str):
        """Log warning message.
        
        Args:
            message: Message to log
        """
        self.logger.warning(message)
    
    def error(self, message: str):
        """Log error message.
        
        Args:
            message: Message to log
        """
        self.logger.error(message)


class ConfigManager:
    """Configuration management utility."""
    
    def __init__(self, config_file: Optional[str] = None):
        """Initialize configuration manager.
        
        Args:
            config_file: Path to configuration file
        """
        self.config_file = config_file
        self._config = {}
        if config_file:
            self.load_config()
    
    def load_config(self):
        """Load configuration from file."""
        if not self.config_file:
            return
        
        try:
            with open(self.config_file, 'r') as f:
                self._config = json.load(f)
        except FileNotFoundError:
            self._config = {}
        except json.JSONDecodeError as e:
            raise ValueError(f"Invalid JSON in config file: {e}")
    
    def get(self, key: str, default: Any = None) -> Any:
        """Get configuration value by key.
        
        Args:
            key: Configuration key
            default: Default value if key not found
            
        Returns:
            Configuration value or default
        """
        return self._config.get(key, default)
    
    def set(self, key: str, value: Any):
        """Set configuration value.
        
        Args:
            key: Configuration key
            value: Value to set
        """
        self._config[key] = value
    
    def save_config(self):
        """Save configuration to file."""
        if not self.config_file:
            return
        
        with open(self.config_file, 'w') as f:
            json.dump(self._config, f, indent=2)


def retry_on_failure(max_retries: int = 3, delay: float = 1.0):
    """Decorator for retrying functions on failure.
    
    Args:
        max_retries: Maximum number of retry attempts
        delay: Delay between retries in seconds
        
    Returns:
        Decorator function
    """
    def decorator(func):
        def wrapper(*args, **kwargs):
            last_exception = None
            
            for attempt in range(max_retries + 1):
                try:
                    return func(*args, **kwargs)
                except Exception as e:
                    last_exception = e
                    if attempt < max_retries:
                        print(f"Attempt {attempt + 1} failed, retrying in {delay}s...")
                        import time
                        time.sleep(delay)
                    
            raise last_exception
        return wrapper
    return decorator


@retry_on_failure(max_retries=2, delay=0.5)
def unreliable_network_call(url: str) -> str:
    """Simulate an unreliable network call.
    
    Args:
        url: URL to call
        
    Returns:
        Response data
    """
    import random
    if random.random() < 0.7:  # 70% chance of failure
        raise ConnectionError("Network timeout")
    return f"Data from {url}"


def benchmark_function(func):
    """Decorator to benchmark function execution time.
    
    Args:
        func: Function to benchmark
        
    Returns:
        Decorated function
    """
    def wrapper(*args, **kwargs):
        start_time = datetime.now()
        result = func(*args, **kwargs)
        end_time = datetime.now()
        
        execution_time = (end_time - start_time).total_seconds()
        print(f"{func.__name__} executed in {execution_time:.4f} seconds")
        
        return result
    return wrapper