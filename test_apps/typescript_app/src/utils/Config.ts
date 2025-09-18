/**
 * Configuration management utility
 */
import * as fs from 'fs';
import * as path from 'path';

export interface ConfigValue {
  [key: string]: any;
}

export class Config {
  private config: ConfigValue = {};
  private configFile?: string;
  private watchers: Map<string, ((value: any) => void)[]> = new Map();

  constructor(configFile?: string) {
    this.configFile = configFile;
    if (configFile) {
      this.loadFromFile();
    }
  }

  public get(key: string, defaultValue?: any): any {
    const keys = key.split('.');
    let current = this.config;

    for (const k of keys) {
      if (current === null || current === undefined || typeof current !== 'object') {
        return defaultValue;
      }
      current = current[k];
    }

    return current !== undefined ? current : defaultValue;
  }

  public set(key: string, value: any): void {
    const keys = key.split('.');
    let current = this.config;

    for (let i = 0; i < keys.length - 1; i++) {
      const k = keys[i];
      if (current[k] === undefined || typeof current[k] !== 'object') {
        current[k] = {};
      }
      current = current[k];
    }

    const lastKey = keys[keys.length - 1];
    current[lastKey] = value;

    this.notifyWatchers(key, value);
  }

  public has(key: string): boolean {
    return this.get(key) !== undefined;
  }

  public remove(key: string): boolean {
    const keys = key.split('.');
    let current = this.config;

    for (let i = 0; i < keys.length - 1; i++) {
      const k = keys[i];
      if (current === null || current === undefined || typeof current !== 'object') {
        return false;
      }
      current = current[k];
    }

    const lastKey = keys[keys.length - 1];
    if (current && typeof current === 'object' && lastKey in current) {
      delete current[lastKey];
      return true;
    }

    return false;
  }

  public merge(other: ConfigValue): void {
    this.config = this.deepMerge(this.config, other);
  }

  public getAll(): ConfigValue {
    return JSON.parse(JSON.stringify(this.config)); // Deep copy
  }

  public clear(): void {
    this.config = {};
  }

  public loadFromFile(filePath?: string): void {
    const configPath = filePath || this.configFile;
    if (!configPath) {
      throw new Error('No config file specified');
    }

    if (!fs.existsSync(configPath)) {
      throw new Error(`Config file not found: ${configPath}`);
    }

    try {
      const content = fs.readFileSync(configPath, 'utf-8');
      const parsed = JSON.parse(content);
      this.config = parsed;
    } catch (error) {
      throw new Error(`Failed to parse config file: ${error}`);
    }
  }

  public saveToFile(filePath?: string): void {
    const configPath = filePath || this.configFile;
    if (!configPath) {
      throw new Error('No config file specified');
    }

    try {
      const dir = path.dirname(configPath);
      if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
      }

      const content = JSON.stringify(this.config, null, 2);
      fs.writeFileSync(configPath, content, 'utf-8');
    } catch (error) {
      throw new Error(`Failed to save config file: ${error}`);
    }
  }

  public watch(key: string, callback: (value: any) => void): void {
    if (!this.watchers.has(key)) {
      this.watchers.set(key, []);
    }
    this.watchers.get(key)!.push(callback);
  }

  public unwatch(key: string, callback?: (value: any) => void): void {
    if (!this.watchers.has(key)) {
      return;
    }

    if (callback) {
      const callbacks = this.watchers.get(key)!;
      const index = callbacks.indexOf(callback);
      if (index > -1) {
        callbacks.splice(index, 1);
      }
    } else {
      this.watchers.delete(key);
    }
  }

  private notifyWatchers(key: string, value: any): void {
    const callbacks = this.watchers.get(key);
    if (callbacks) {
      callbacks.forEach(callback => {
        try {
          callback(value);
        } catch (error) {
          console.error(`Config watcher error for key '${key}':`, error);
        }
      });
    }
  }

  private deepMerge(target: any, source: any): any {
    if (Array.isArray(source)) {
      return [...source];
    }

    if (source === null || typeof source !== 'object') {
      return source;
    }

    const result = Array.isArray(target) ? [] : { ...target };

    for (const key in source) {
      if (source.hasOwnProperty(key)) {
        if (typeof source[key] === 'object' && source[key] !== null && !Array.isArray(source[key])) {
          result[key] = this.deepMerge(result[key] || {}, source[key]);
        } else {
          result[key] = source[key];
        }
      }
    }

    return result;
  }
}

// Utility functions
export function createConfig(data?: ConfigValue): Config {
  const config = new Config();
  if (data) {
    config.merge(data);
  }
  return config;
}

export function loadConfigFromFile(filePath: string): Config {
  return new Config(filePath);
}

export function createEnvironmentConfig(prefix: string = ''): Config {
  const config = new Config();
  
  Object.keys(process.env).forEach(key => {
    if (!prefix || key.startsWith(prefix)) {
      const configKey = prefix ? key.substring(prefix.length) : key;
      const value = process.env[key];
      
      // Try to parse as JSON, fall back to string
      try {
        config.set(configKey.toLowerCase(), JSON.parse(value!));
      } catch {
        config.set(configKey.toLowerCase(), value);
      }
    }
  });
  
  return config;
}