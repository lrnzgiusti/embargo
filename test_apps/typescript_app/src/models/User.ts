/**
 * User models and interfaces
 */
import { Entity, Identifiable } from './Base';
import { hashPassword, validateEmail } from '../utils/crypto';

export interface UserData {
  id?: string;
  username: string;
  email: string;
  isActive: boolean;
  createdAt: Date;
  updatedAt: Date;
}

export interface AdminPermissions {
  read: boolean;
  write: boolean;
  delete: boolean;
  admin: boolean;
}

export abstract class BaseUser implements Entity, Identifiable {
  public readonly id: string;
  public username: string;
  public email: string;
  public isActive: boolean;
  public createdAt: Date;
  public updatedAt: Date;
  protected passwordHash?: string;

  constructor(username: string, email: string) {
    this.id = this.generateId();
    this.username = username;
    this.email = email;
    this.isActive = true;
    this.createdAt = new Date();
    this.updatedAt = new Date();

    this.validateInput();
  }

  public abstract toJSON(): UserData;
  
  public abstract getType(): string;

  protected validateInput(): void {
    if (!this.username || this.username.length < 3) {
      throw new Error('Username must be at least 3 characters long');
    }

    if (!validateEmail(this.email)) {
      throw new Error('Invalid email format');
    }
  }

  protected generateId(): string {
    return `user_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  public setPassword(password: string): void {
    if (password.length < 8) {
      throw new Error('Password must be at least 8 characters long');
    }
    this.passwordHash = hashPassword(password);
    this.updateTimestamp();
  }

  public activate(): void {
    this.isActive = true;
    this.updateTimestamp();
  }

  public deactivate(): void {
    this.isActive = false;
    this.updateTimestamp();
  }

  protected updateTimestamp(): void {
    this.updatedAt = new Date();
  }

  public getId(): string {
    return this.id;
  }
}

export class User extends BaseUser {
  constructor(username: string, email: string) {
    super(username, email);
  }

  public toJSON(): UserData {
    return {
      id: this.id,
      username: this.username,
      email: this.email,
      isActive: this.isActive,
      createdAt: this.createdAt,
      updatedAt: this.updatedAt,
    };
  }

  public getType(): string {
    return 'user';
  }

  public updateProfile(username?: string, email?: string): void {
    if (username) {
      this.username = username;
    }
    
    if (email) {
      if (!validateEmail(email)) {
        throw new Error('Invalid email format');
      }
      this.email = email;
    }

    this.updateTimestamp();
  }
}

export class AdminUser extends User {
  private permissions: string[];
  public readonly isAdmin: boolean = true;

  constructor(username: string, email: string, permissions: string[] = []) {
    super(username, email);
    this.permissions = permissions.length > 0 ? permissions : ['read', 'write'];
  }

  public toJSON(): UserData & { isAdmin: boolean; permissions: string[] } {
    return {
      ...super.toJSON(),
      isAdmin: this.isAdmin,
      permissions: [...this.permissions], // Return a copy
    };
  }

  public getType(): string {
    return 'admin';
  }

  public getPermissions(): string[] {
    return [...this.permissions]; // Return a copy to prevent mutation
  }

  public hasPermission(permission: string): boolean {
    return this.permissions.includes(permission);
  }

  public grantPermission(permission: string): void {
    if (!this.permissions.includes(permission)) {
      this.permissions.push(permission);
      this.updateTimestamp();
    }
  }

  public revokePermission(permission: string): void {
    const index = this.permissions.indexOf(permission);
    if (index > -1) {
      this.permissions.splice(index, 1);
      this.updateTimestamp();
    }
  }

  public setPermissions(permissions: string[]): void {
    this.permissions = [...permissions];
    this.updateTimestamp();
  }
}

export class GuestUser implements Entity {
  public readonly id: string;
  public readonly username: string = 'guest';
  public readonly isActive: boolean = true;
  public readonly createdAt: Date;

  constructor() {
    this.id = `guest_${Date.now()}`;
    this.createdAt = new Date();
  }

  public toJSON(): Partial<UserData> {
    return {
      id: this.id,
      username: this.username,
      isActive: this.isActive,
      createdAt: this.createdAt,
    };
  }

  public getType(): string {
    return 'guest';
  }
}

// Utility functions for user operations
export function createUser(type: 'user' | 'admin' | 'guest', 
                          username?: string, 
                          email?: string, 
                          permissions?: string[]): BaseUser | GuestUser {
  switch (type) {
    case 'user':
      if (!username || !email) {
        throw new Error('Username and email are required for regular users');
      }
      return new User(username, email);
    
    case 'admin':
      if (!username || !email) {
        throw new Error('Username and email are required for admin users');
      }
      return new AdminUser(username, email, permissions);
    
    case 'guest':
      return new GuestUser();
    
    default:
      throw new Error(`Unknown user type: ${type}`);
  }
}

export function isAdminUser(user: BaseUser | GuestUser): user is AdminUser {
  return user instanceof AdminUser;
}

export function isRegularUser(user: BaseUser | GuestUser): user is User {
  return user instanceof User && !(user instanceof AdminUser);
}

export function isGuestUser(user: BaseUser | GuestUser): user is GuestUser {
  return user instanceof GuestUser;
}