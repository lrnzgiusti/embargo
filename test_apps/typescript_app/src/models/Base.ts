/**
 * Base interfaces and abstract classes
 */

export interface Entity {
  id: string;
  createdAt: Date;
  toJSON(): any;
  getType(): string;
}

export interface Identifiable {
  getId(): string;
}

export interface Timestamped {
  createdAt: Date;
  updatedAt: Date;
}

export interface Activatable {
  isActive: boolean;
  activate(): void;
  deactivate(): void;
}

export abstract class BaseEntity implements Entity, Timestamped {
  public readonly id: string;
  public readonly createdAt: Date;
  public updatedAt: Date;

  constructor() {
    this.id = this.generateId();
    this.createdAt = new Date();
    this.updatedAt = new Date();
  }

  public abstract toJSON(): any;
  public abstract getType(): string;

  protected abstract generateId(): string;

  protected updateTimestamp(): void {
    this.updatedAt = new Date();
  }
}

export enum EntityStatus {
  ACTIVE = 'active',
  INACTIVE = 'inactive',
  PENDING = 'pending',
  SUSPENDED = 'suspended',
  DELETED = 'deleted',
}

export interface Repository<T extends Entity> {
  save(entity: T): Promise<T>;
  findById(id: string): Promise<T | null>;
  findAll(): Promise<T[]>;
  update(entity: T): Promise<T>;
  delete(id: string): Promise<boolean>;
}

export abstract class BaseRepository<T extends Entity> implements Repository<T> {
  protected items: Map<string, T> = new Map();

  public async save(entity: T): Promise<T> {
    this.items.set(entity.id, entity);
    return entity;
  }

  public async findById(id: string): Promise<T | null> {
    return this.items.get(id) || null;
  }

  public async findAll(): Promise<T[]> {
    return Array.from(this.items.values());
  }

  public async update(entity: T): Promise<T> {
    if (!this.items.has(entity.id)) {
      throw new Error(`Entity with id ${entity.id} not found`);
    }
    this.items.set(entity.id, entity);
    return entity;
  }

  public async delete(id: string): Promise<boolean> {
    return this.items.delete(id);
  }

  public async count(): Promise<number> {
    return this.items.size;
  }

  protected async exists(id: string): Promise<boolean> {
    return this.items.has(id);
  }
}