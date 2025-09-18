/**
 * Main application service
 */
import { Config } from '../utils/Config';
import { Logger } from '../utils/Logger';
import { UserService } from './UserService';
import { DatabaseService } from './DatabaseService';
import { EventEmitter } from 'events';

export interface ApplicationOptions {
  environment?: string;
  debug?: boolean;
  maxRetries?: number;
}

export class Application extends EventEmitter {
  private config: Config;
  private logger: Logger;
  private userService: UserService;
  private databaseService: DatabaseService;
  private isInitialized: boolean = false;
  private startTime: Date;

  constructor(config: Config, logger: Logger, options?: ApplicationOptions) {
    super();
    
    this.config = config;
    this.logger = logger;
    this.startTime = new Date();
    
    this.initializeServices();
    this.setupEventHandlers();
  }

  public async initialize(): Promise<void> {
    if (this.isInitialized) {
      this.logger.warn('Application already initialized');
      return;
    }

    try {
      this.logger.info('Initializing application...');
      
      await this.initializeDatabase();
      await this.initializeServices();
      
      this.isInitialized = true;
      this.emit('initialized');
      
      this.logger.info('Application initialized successfully');
    } catch (error) {
      this.logger.error(`Failed to initialize application: ${error}`);
      this.emit('error', error);
      throw error;
    }
  }

  public async shutdown(): Promise<void> {
    this.logger.info('Shutting down application...');
    
    try {
      await this.databaseService.disconnect();
      this.removeAllListeners();
      
      this.logger.info('Application shut down successfully');
      this.emit('shutdown');
    } catch (error) {
      this.logger.error(`Error during shutdown: ${error}`);
      throw error;
    }
  }

  public getUptime(): number {
    return Date.now() - this.startTime.getTime();
  }

  public getStatus(): ApplicationStatus {
    return {
      isInitialized: this.isInitialized,
      uptime: this.getUptime(),
      userCount: this.userService.getUserCount(),
      databaseConnected: this.databaseService.isConnected(),
    };
  }

  private async initializeDatabase(): Promise<void> {
    const dbConfig = this.config.get('database');
    await this.databaseService.connect(dbConfig);
  }

  private initializeServices(): void {
    this.databaseService = new DatabaseService(this.logger);
    this.userService = new UserService(this.logger);
  }

  private setupEventHandlers(): void {
    this.on('error', (error: Error) => {
      this.logger.error(`Application error: ${error.message}`);
    });

    this.on('user:created', (userId: string) => {
      this.logger.info(`User created: ${userId}`);
    });

    this.on('user:deleted', (userId: string) => {
      this.logger.info(`User deleted: ${userId}`);
    });

    // Handle process signals
    process.on('SIGTERM', () => {
      this.logger.info('Received SIGTERM, shutting down gracefully');
      this.shutdown().then(() => process.exit(0));
    });

    process.on('SIGINT', () => {
      this.logger.info('Received SIGINT, shutting down gracefully');
      this.shutdown().then(() => process.exit(0));
    });
  }

  public getUserService(): UserService {
    return this.userService;
  }

  public getDatabaseService(): DatabaseService {
    return this.databaseService;
  }

  public getConfig(): Config {
    return this.config;
  }

  public getLogger(): Logger {
    return this.logger;
  }
}

export interface ApplicationStatus {
  isInitialized: boolean;
  uptime: number;
  userCount: number;
  databaseConnected: boolean;
}

export class ApplicationBuilder {
  private config?: Config;
  private logger?: Logger;
  private options?: ApplicationOptions;

  public setConfig(config: Config): ApplicationBuilder {
    this.config = config;
    return this;
  }

  public setLogger(logger: Logger): ApplicationBuilder {
    this.logger = logger;
    return this;
  }

  public setOptions(options: ApplicationOptions): ApplicationBuilder {
    this.options = options;
    return this;
  }

  public build(): Application {
    if (!this.config) {
      throw new Error('Config is required');
    }
    
    if (!this.logger) {
      this.logger = new Logger('Application');
    }

    return new Application(this.config, this.logger, this.options);
  }
}