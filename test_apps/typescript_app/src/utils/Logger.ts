/**
 * Logging utility classes
 */

export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3,
}

export interface LogEntry {
  timestamp: Date;
  level: LogLevel;
  message: string;
  context?: string;
  metadata?: Record<string, any>;
}

export interface LoggerOptions {
  level?: LogLevel;
  context?: string;
  enableConsole?: boolean;
  enableFile?: boolean;
  filename?: string;
}

export abstract class BaseLogger {
  protected level: LogLevel;
  protected context: string;

  constructor(context: string, level: LogLevel = LogLevel.INFO) {
    this.context = context;
    this.level = level;
  }

  public abstract log(level: LogLevel, message: string, metadata?: Record<string, any>): void;

  public debug(message: string, metadata?: Record<string, any>): void {
    this.log(LogLevel.DEBUG, message, metadata);
  }

  public info(message: string, metadata?: Record<string, any>): void {
    this.log(LogLevel.INFO, message, metadata);
  }

  public warn(message: string, metadata?: Record<string, any>): void {
    this.log(LogLevel.WARN, message, metadata);
  }

  public error(message: string, metadata?: Record<string, any>): void {
    this.log(LogLevel.ERROR, message, metadata);
  }

  protected shouldLog(level: LogLevel): boolean {
    return level >= this.level;
  }

  protected formatMessage(level: LogLevel, message: string): string {
    const timestamp = new Date().toISOString();
    const levelName = LogLevel[level];
    return `[${timestamp}] ${levelName} [${this.context}]: ${message}`;
  }
}

export class Logger extends BaseLogger {
  private enableConsole: boolean;
  private enableFile: boolean;
  private filename?: string;
  private logHistory: LogEntry[] = [];

  constructor(context: string, options?: LoggerOptions) {
    super(context, options?.level || LogLevel.INFO);
    
    this.enableConsole = options?.enableConsole !== false;
    this.enableFile = options?.enableFile || false;
    this.filename = options?.filename;
  }

  public log(level: LogLevel, message: string, metadata?: Record<string, any>): void {
    if (!this.shouldLog(level)) {
      return;
    }

    const logEntry: LogEntry = {
      timestamp: new Date(),
      level,
      message,
      context: this.context,
      metadata,
    };

    this.logHistory.push(logEntry);

    if (this.enableConsole) {
      this.logToConsole(logEntry);
    }

    if (this.enableFile && this.filename) {
      this.logToFile(logEntry);
    }
  }

  public setLevel(level: LogLevel): void {
    this.level = level;
  }

  public getLevel(): LogLevel {
    return this.level;
  }

  public getHistory(): LogEntry[] {
    return [...this.logHistory];
  }

  public clearHistory(): void {
    this.logHistory = [];
  }

  public createChild(context: string): Logger {
    return new Logger(`${this.context}.${context}`, {
      level: this.level,
      enableConsole: this.enableConsole,
      enableFile: this.enableFile,
      filename: this.filename,
    });
  }

  private logToConsole(entry: LogEntry): void {
    const formattedMessage = this.formatMessage(entry.level, entry.message);
    const metadataStr = entry.metadata ? ` ${JSON.stringify(entry.metadata)}` : '';
    
    switch (entry.level) {
      case LogLevel.DEBUG:
        console.debug(formattedMessage + metadataStr);
        break;
      case LogLevel.INFO:
        console.info(formattedMessage + metadataStr);
        break;
      case LogLevel.WARN:
        console.warn(formattedMessage + metadataStr);
        break;
      case LogLevel.ERROR:
        console.error(formattedMessage + metadataStr);
        break;
    }
  }

  private logToFile(entry: LogEntry): void {
    // In a real implementation, this would write to a file
    // For this example, we'll just simulate it
    const formattedMessage = this.formatMessage(entry.level, entry.message);
    const metadataStr = entry.metadata ? ` ${JSON.stringify(entry.metadata)}` : '';
    
    // Simulate file writing
    setTimeout(() => {
      // fs.appendFileSync(this.filename!, formattedMessage + metadataStr + '\n');
    }, 0);
  }
}

export class BufferedLogger extends BaseLogger {
  private buffer: LogEntry[] = [];
  private bufferSize: number;
  private flushCallback?: (entries: LogEntry[]) => void;

  constructor(context: string, bufferSize: number = 100, level: LogLevel = LogLevel.INFO) {
    super(context, level);
    this.bufferSize = bufferSize;
  }

  public log(level: LogLevel, message: string, metadata?: Record<string, any>): void {
    if (!this.shouldLog(level)) {
      return;
    }

    const logEntry: LogEntry = {
      timestamp: new Date(),
      level,
      message,
      context: this.context,
      metadata,
    };

    this.buffer.push(logEntry);

    if (this.buffer.length >= this.bufferSize) {
      this.flush();
    }
  }

  public setFlushCallback(callback: (entries: LogEntry[]) => void): void {
    this.flushCallback = callback;
  }

  public flush(): void {
    if (this.buffer.length === 0) {
      return;
    }

    if (this.flushCallback) {
      this.flushCallback([...this.buffer]);
    } else {
      // Default behavior: log to console
      this.buffer.forEach(entry => {
        const formattedMessage = this.formatMessage(entry.level, entry.message);
        console.log(formattedMessage);
      });
    }

    this.buffer = [];
  }

  public getBufferSize(): number {
    return this.buffer.length;
  }

  public setBufferSize(size: number): void {
    this.bufferSize = size;
  }
}

// Utility functions
export function createLogger(context: string, level?: LogLevel): Logger {
  return new Logger(context, { level });
}

export function createFileLogger(context: string, filename: string, level?: LogLevel): Logger {
  return new Logger(context, {
    level,
    enableFile: true,
    filename,
  });
}

export function createBufferedLogger(context: string, bufferSize?: number, level?: LogLevel): BufferedLogger {
  return new BufferedLogger(context, bufferSize, level);
}

// Decorator for method logging
export function LogMethod(level: LogLevel = LogLevel.DEBUG) {
  return function (target: any, propertyName: string, descriptor: PropertyDescriptor) {
    const method = descriptor.value;
    const className = target.constructor.name;

    descriptor.value = function (...args: any[]) {
      const logger = new Logger(`${className}.${propertyName}`);
      
      logger.log(level, `Method called with args: ${JSON.stringify(args)}`);
      
      const startTime = Date.now();
      const result = method.apply(this, args);
      const endTime = Date.now();
      
      if (result instanceof Promise) {
        return result
          .then((value) => {
            logger.log(level, `Method completed in ${endTime - startTime}ms`);
            return value;
          })
          .catch((error) => {
            logger.error(`Method failed in ${endTime - startTime}ms: ${error.message}`);
            throw error;
          });
      } else {
        logger.log(level, `Method completed in ${endTime - startTime}ms`);
        return result;
      }
    };

    return descriptor;
  };
}