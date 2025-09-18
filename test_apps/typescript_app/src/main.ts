/**
 * Main application entry point
 */
import { Application } from './services/Application';
import { UserService } from './services/UserService';
import { Logger } from './utils/Logger';
import { Config } from './utils/Config';
import { User, AdminUser } from './models/User';

interface AppConfig {
  environment: string;
  database: {
    host: string;
    port: number;
    name: string;
  };
  logging: {
    level: string;
    file: string;
  };
}

class MainApplication {
  private app: Application;
  private logger: Logger;
  private config: Config;

  constructor() {
    this.logger = new Logger('MainApplication');
    this.config = new Config('config.json');
    this.app = new Application(this.config, this.logger);
  }

  public async run(): Promise<number> {
    try {
      this.logger.info('Starting application');
      
      await this.initializeApplication();
      await this.runMainLoop();
      
      this.logger.info('Application completed successfully');
      return 0;
    } catch (error) {
      this.logger.error(`Application failed: ${error}`);
      return 1;
    }
  }

  private async initializeApplication(): Promise<void> {
    await this.app.initialize();
  }

  private async runMainLoop(): Promise<void> {
    const userService = new UserService();
    
    // Create sample users
    const users = this.createSampleUsers();
    
    for (const user of users) {
      await userService.createUser(user);
      this.logger.debug(`Created user: ${user.username}`);
    }

    // Process users
    const allUsers = await userService.getAllUsers();
    this.processUsers(allUsers);
  }

  private createSampleUsers(): User[] {
    const users: User[] = [];
    
    users.push(new User('alice', 'alice@example.com'));
    users.push(new User('bob', 'bob@example.com'));
    users.push(new AdminUser('charlie', 'charlie@example.com', ['read', 'write', 'admin']));
    
    return users;
  }

  private processUsers(users: User[]): void {
    users.forEach((user, index) => {
      this.logger.info(`Processing user ${index + 1}: ${user.username}`);
      
      if (user instanceof AdminUser) {
        this.logger.info(`Admin user detected with permissions: ${user.getPermissions().join(', ')}`);
      }
      
      const userData = user.toJSON();
      this.logger.debug(`User data: ${JSON.stringify(userData, null, 2)}`);
    });
  }
}

// Utility functions
export function formatUptime(milliseconds: number): string {
  const seconds = Math.floor(milliseconds / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  
  return `${hours}h ${minutes % 60}m ${seconds % 60}s`;
}

export function validateEnvironment(): boolean {
  const requiredEnvVars = ['NODE_ENV', 'DATABASE_URL'];
  
  for (const envVar of requiredEnvVars) {
    if (!process.env[envVar]) {
      console.error(`Missing required environment variable: ${envVar}`);
      return false;
    }
  }
  
  return true;
}

// Main execution
async function main(): Promise<void> {
  if (!validateEnvironment()) {
    process.exit(1);
  }

  const mainApp = new MainApplication();
  const exitCode = await mainApp.run();
  
  process.exit(exitCode);
}

// Only run if this file is the main module
if (require.main === module) {
  main().catch(error => {
    console.error('Unhandled error:', error);
    process.exit(1);
  });
}