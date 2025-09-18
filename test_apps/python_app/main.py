"""
Main application module demonstrating various Python language constructs.
"""
import os
import sys
from datetime import datetime
from typing import List, Dict, Optional

from models import User, UserManager, DatabaseError
from utils import Logger, calculate_hash, format_output


class Application:
    """Main application class that orchestrates the entire system."""
    
    def __init__(self, config_path: str):
        """Initialize the application with configuration.
        
        Args:
            config_path: Path to configuration file
        """
        self.config_path = config_path
        self.logger = Logger("application")
        self.user_manager = UserManager()
        self.start_time = datetime.now()
    
    def run(self) -> int:
        """Run the main application loop.
        
        Returns:
            Exit code (0 for success, non-zero for error)
        """
        try:
            self.logger.info("Starting application")
            self._initialize_system()
            self._process_users()
            return 0
        except Exception as e:
            self.logger.error(f"Application failed: {e}")
            return 1
    
    def _initialize_system(self):
        """Initialize the system components."""
        config = self._load_config()
        self.user_manager.configure(config)
    
    def _load_config(self) -> Dict[str, str]:
        """Load configuration from file.
        
        Returns:
            Configuration dictionary
        """
        if not os.path.exists(self.config_path):
            raise FileNotFoundError(f"Config file not found: {self.config_path}")
        
        # Simplified config loading
        return {"database_url": "sqlite:///users.db"}
    
    def _process_users(self):
        """Process user data."""
        users = self.user_manager.get_all_users()
        for user in users:
            hash_value = calculate_hash(user.username)
            formatted_data = format_output(user.to_dict())
            self.logger.debug(f"Processed user {user.username} with hash {hash_value}")


def create_sample_users() -> List[User]:
    """Create sample users for testing.
    
    Returns:
        List of sample users
    """
    users = []
    names = ["alice", "bob", "charlie", "diana"]
    
    for name in names:
        user = User(name, f"{name}@example.com")
        users.append(user)
    
    return users


def main():
    """Entry point of the application."""
    app = Application("config.json")
    exit_code = app.run()
    sys.exit(exit_code)


if __name__ == "__main__":
    main()