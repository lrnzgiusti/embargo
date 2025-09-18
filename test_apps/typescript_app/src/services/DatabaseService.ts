/**
 * Database service for managing database connections and operations
 */
import { Logger } from '../utils/Logger';
import { EventEmitter } from 'events';

export interface DatabaseConfig {
  host: string;
  port: number;
  database: string;
  username?: string;
  password?: string;
  ssl?: boolean;
  poolSize?: number;
  timeout?: number;
}

export interface QueryResult<T = any> {
  rows: T[];
  rowCount: number;
  affectedRows: number;
}

export interface Transaction {
  query<T = any>(sql: string, params?: any[]): Promise<QueryResult<T>>;
  commit(): Promise<void>;
  rollback(): Promise<void>;
}

export abstract class BaseDatabaseService extends EventEmitter {
  protected config: DatabaseConfig;
  protected logger: Logger;
  protected connected: boolean = false;

  constructor(logger?: Logger) {
    super();
    this.logger = logger || new Logger('DatabaseService');
  }

  public abstract connect(config: DatabaseConfig): Promise<void>;
  public abstract disconnect(): Promise<void>;
  public abstract query<T = any>(sql: string, params?: any[]): Promise<QueryResult<T>>;
  public abstract beginTransaction(): Promise<Transaction>;

  public isConnected(): boolean {
    return this.connected;
  }

  public getConfig(): DatabaseConfig | undefined {
    return this.config;
  }
}

export class DatabaseService extends BaseDatabaseService {
  private connectionPool: Map<string, Connection> = new Map();
  private transactionId: number = 0;

  constructor(logger?: Logger) {
    super(logger);
    this.setupEventHandlers();
  }

  public async connect(config: DatabaseConfig): Promise<void> {
    try {
      this.logger.info(`Connecting to database: ${config.host}:${config.port}`);
      this.config = config;
      
      // Simulate connection setup
      await this.createConnectionPool();
      
      this.connected = true;
      this.emit('connected', config);
      this.logger.info('Database connected successfully');
    } catch (error) {
      this.logger.error(`Database connection failed: ${error}`);
      this.emit('error', error);
      throw error;
    }
  }

  public async disconnect(): Promise<void> {
    try {
      this.logger.info('Disconnecting from database...');
      
      // Close all connections in the pool
      for (const [id, connection] of this.connectionPool) {
        await connection.close();
        this.connectionPool.delete(id);
      }
      
      this.connected = false;
      this.emit('disconnected');
      this.logger.info('Database disconnected successfully');
    } catch (error) {
      this.logger.error(`Database disconnection failed: ${error}`);
      throw error;
    }
  }

  public async query<T = any>(sql: string, params?: any[]): Promise<QueryResult<T>> {
    if (!this.connected) {
      throw new Error('Database is not connected');
    }

    const connection = this.getConnection();
    
    try {
      this.logger.debug(`Executing query: ${sql.substring(0, 100)}...`);
      const result = await connection.execute<T>(sql, params);
      this.emit('query:executed', { sql, params, result });
      return result;
    } catch (error) {
      this.logger.error(`Query execution failed: ${error}`);
      this.emit('query:error', { sql, params, error });
      throw error;
    }
  }

  public async beginTransaction(): Promise<Transaction> {
    if (!this.connected) {
      throw new Error('Database is not connected');
    }

    const transactionId = ++this.transactionId;
    const connection = this.getConnection();
    
    await connection.execute('BEGIN TRANSACTION');
    this.logger.debug(`Transaction ${transactionId} started`);
    
    return new DatabaseTransaction(transactionId, connection, this.logger);
  }

  public async createTable(tableName: string, schema: TableSchema): Promise<void> {
    const columns = schema.columns.map(col => 
      `${col.name} ${col.type}${col.nullable ? '' : ' NOT NULL'}${col.primaryKey ? ' PRIMARY KEY' : ''}`
    ).join(', ');
    
    const sql = `CREATE TABLE IF NOT EXISTS ${tableName} (${columns})`;
    await this.query(sql);
    
    this.logger.info(`Table '${tableName}' created successfully`);
  }

  public async dropTable(tableName: string): Promise<void> {
    const sql = `DROP TABLE IF EXISTS ${tableName}`;
    await this.query(sql);
    
    this.logger.info(`Table '${tableName}' dropped successfully`);
  }

  private async createConnectionPool(): Promise<void> {
    const poolSize = this.config.poolSize || 10;
    
    for (let i = 0; i < poolSize; i++) {
      const connection = new Connection(`connection_${i}`, this.config);
      await connection.connect();
      this.connectionPool.set(connection.getId(), connection);
    }
    
    this.logger.info(`Connection pool created with ${poolSize} connections`);
  }

  private getConnection(): Connection {
    const connections = Array.from(this.connectionPool.values());
    const availableConnection = connections.find(conn => !conn.inUse);
    
    if (!availableConnection) {
      throw new Error('No available connections in pool');
    }
    
    availableConnection.inUse = true;
    return availableConnection;
  }

  private setupEventHandlers(): void {
    this.on('connected', (config: DatabaseConfig) => {
      this.logger.info(`Connected to ${config.host}:${config.port}/${config.database}`);
    });

    this.on('disconnected', () => {
      this.logger.info('Database connection closed');
    });

    this.on('error', (error: Error) => {
      this.logger.error(`Database error: ${error.message}`);
    });
  }
}

class Connection {
  public inUse: boolean = false;
  private id: string;
  private config: DatabaseConfig;
  private connected: boolean = false;

  constructor(id: string, config: DatabaseConfig) {
    this.id = id;
    this.config = config;
  }

  public async connect(): Promise<void> {
    // Simulate connection
    await this.delay(100);
    this.connected = true;
  }

  public async close(): Promise<void> {
    // Simulate connection close
    await this.delay(50);
    this.connected = false;
    this.inUse = false;
  }

  public async execute<T = any>(sql: string, params?: any[]): Promise<QueryResult<T>> {
    if (!this.connected) {
      throw new Error('Connection is not established');
    }

    // Simulate query execution
    await this.delay(Math.random() * 100);
    
    return {
      rows: [] as T[],
      rowCount: 0,
      affectedRows: 0,
    };
  }

  public getId(): string {
    return this.id;
  }

  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

class DatabaseTransaction implements Transaction {
  private transactionId: number;
  private connection: Connection;
  private logger: Logger;
  private committed: boolean = false;
  private rolledBack: boolean = false;

  constructor(transactionId: number, connection: Connection, logger: Logger) {
    this.transactionId = transactionId;
    this.connection = connection;
    this.logger = logger;
  }

  public async query<T = any>(sql: string, params?: any[]): Promise<QueryResult<T>> {
    if (this.committed || this.rolledBack) {
      throw new Error('Transaction is no longer active');
    }

    this.logger.debug(`Transaction ${this.transactionId} executing: ${sql.substring(0, 100)}...`);
    return await this.connection.execute<T>(sql, params);
  }

  public async commit(): Promise<void> {
    if (this.committed || this.rolledBack) {
      throw new Error('Transaction is no longer active');
    }

    await this.connection.execute('COMMIT');
    this.committed = true;
    this.connection.inUse = false;
    
    this.logger.debug(`Transaction ${this.transactionId} committed`);
  }

  public async rollback(): Promise<void> {
    if (this.committed || this.rolledBack) {
      throw new Error('Transaction is no longer active');
    }

    await this.connection.execute('ROLLBACK');
    this.rolledBack = true;
    this.connection.inUse = false;
    
    this.logger.debug(`Transaction ${this.transactionId} rolled back`);
  }
}

export interface TableSchema {
  columns: ColumnDefinition[];
}

export interface ColumnDefinition {
  name: string;
  type: string;
  nullable?: boolean;
  primaryKey?: boolean;
  unique?: boolean;
  defaultValue?: any;
}