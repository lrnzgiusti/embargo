/**
 * User management service
 */
import { User, AdminUser, BaseUser, createUser, isAdminUser } from '../models/User';
import { BaseRepository } from '../models/Base';
import { Logger } from '../utils/Logger';
import { validateEmail, hashPassword } from '../utils/crypto';
import { EventEmitter } from 'events';

export interface UserCreateRequest {
  username: string;
  email: string;
  password: string;
  type?: 'user' | 'admin';
  permissions?: string[];
}

export interface UserUpdateRequest {
  username?: string;
  email?: string;
  isActive?: boolean;
  permissions?: string[];
}

export interface UserSearchCriteria {
  username?: string;
  email?: string;
  isActive?: boolean;
  type?: string;
}

class UserRepository extends BaseRepository<BaseUser> {
  public async findByUsername(username: string): Promise<BaseUser | null> {
    const users = Array.from(this.items.values());
    return users.find(user => user.username === username) || null;
  }

  public async findByEmail(email: string): Promise<BaseUser | null> {
    const users = Array.from(this.items.values());
    return users.find(user => user.email === email) || null;
  }

  public async findByType(type: string): Promise<BaseUser[]> {
    const users = Array.from(this.items.values());
    return users.filter(user => user.getType() === type);
  }

  public async search(criteria: UserSearchCriteria): Promise<BaseUser[]> {
    const users = Array.from(this.items.values());
    
    return users.filter(user => {
      if (criteria.username && user.username !== criteria.username) {
        return false;
      }
      if (criteria.email && user.email !== criteria.email) {
        return false;
      }
      if (criteria.isActive !== undefined && user.isActive !== criteria.isActive) {
        return false;
      }
      if (criteria.type && user.getType() !== criteria.type) {
        return false;
      }
      return true;
    });
  }
}

export class UserService extends EventEmitter {
  private repository: UserRepository;
  private logger: Logger;

  constructor(logger?: Logger) {
    super();
    this.repository = new UserRepository();
    this.logger = logger || new Logger('UserService');
  }

  public async createUser(request: UserCreateRequest): Promise<BaseUser> {
    this.validateCreateRequest(request);
    
    // Check if user already exists
    const existingUser = await this.repository.findByUsername(request.username);
    if (existingUser) {
      throw new Error(`User with username '${request.username}' already exists`);
    }

    const existingEmail = await this.repository.findByEmail(request.email);
    if (existingEmail) {
      throw new Error(`User with email '${request.email}' already exists`);
    }

    let user: BaseUser;
    
    if (request.type === 'admin') {
      user = new AdminUser(request.username, request.email, request.permissions);
      this.logger.info(`Creating admin user: ${request.username}`);
    } else {
      user = new User(request.username, request.email);
      this.logger.info(`Creating regular user: ${request.username}`);
    }

    user.setPassword(request.password);
    
    const savedUser = await this.repository.save(user);
    this.emit('user:created', savedUser);
    
    return savedUser;
  }

  public async createUser(user: BaseUser): Promise<BaseUser> {
    const existingUser = await this.repository.findByUsername(user.username);
    if (existingUser) {
      throw new Error(`User with username '${user.username}' already exists`);
    }

    const savedUser = await this.repository.save(user);
    this.emit('user:created', savedUser);
    
    return savedUser;
  }

  public async getUserById(id: string): Promise<BaseUser | null> {
    return await this.repository.findById(id);
  }

  public async getUserByUsername(username: string): Promise<BaseUser | null> {
    return await this.repository.findByUsername(username);
  }

  public async getUserByEmail(email: string): Promise<BaseUser | null> {
    return await this.repository.findByEmail(email);
  }

  public async getAllUsers(): Promise<BaseUser[]> {
    return await this.repository.findAll();
  }

  public async getActiveUsers(): Promise<BaseUser[]> {
    return await this.repository.search({ isActive: true });
  }

  public async getAdminUsers(): Promise<AdminUser[]> {
    const users = await this.repository.findByType('admin');
    return users.filter(isAdminUser);
  }

  public async updateUser(id: string, updates: UserUpdateRequest): Promise<BaseUser> {
    const user = await this.repository.findById(id);
    if (!user) {
      throw new Error(`User with id '${id}' not found`);
    }

    if (updates.username !== undefined) {
      user.username = updates.username;
    }

    if (updates.email !== undefined) {
      if (!validateEmail(updates.email)) {
        throw new Error('Invalid email format');
      }
      user.email = updates.email;
    }

    if (updates.isActive !== undefined) {
      if (updates.isActive) {
        user.activate();
      } else {
        user.deactivate();
      }
    }

    if (updates.permissions !== undefined && isAdminUser(user)) {
      user.setPermissions(updates.permissions);
    }

    const updatedUser = await this.repository.update(user);
    this.emit('user:updated', updatedUser);
    
    return updatedUser;
  }

  public async deleteUser(id: string): Promise<boolean> {
    const user = await this.repository.findById(id);
    if (!user) {
      return false;
    }

    const deleted = await this.repository.delete(id);
    if (deleted) {
      this.emit('user:deleted', user);
    }
    
    return deleted;
  }

  public async searchUsers(criteria: UserSearchCriteria): Promise<BaseUser[]> {
    return await this.repository.search(criteria);
  }

  public async getUserCount(): Promise<number> {
    return await this.repository.count();
  }

  public getUserCount(): number {
    return this.repository['items'].size; // Access private field for sync version
  }

  public async activateUser(id: string): Promise<BaseUser> {
    return await this.updateUser(id, { isActive: true });
  }

  public async deactivateUser(id: string): Promise<BaseUser> {
    return await this.updateUser(id, { isActive: false });
  }

  public async grantPermission(userId: string, permission: string): Promise<AdminUser> {
    const user = await this.repository.findById(userId);
    if (!user) {
      throw new Error(`User with id '${userId}' not found`);
    }

    if (!isAdminUser(user)) {
      throw new Error('User is not an admin user');
    }

    user.grantPermission(permission);
    await this.repository.update(user);
    this.emit('permission:granted', { userId, permission });
    
    return user;
  }

  public async revokePermission(userId: string, permission: string): Promise<AdminUser> {
    const user = await this.repository.findById(userId);
    if (!user) {
      throw new Error(`User with id '${userId}' not found`);
    }

    if (!isAdminUser(user)) {
      throw new Error('User is not an admin user');
    }

    user.revokePermission(permission);
    await this.repository.update(user);
    this.emit('permission:revoked', { userId, permission });
    
    return user;
  }

  private validateCreateRequest(request: UserCreateRequest): void {
    if (!request.username || request.username.length < 3) {
      throw new Error('Username must be at least 3 characters long');
    }

    if (!validateEmail(request.email)) {
      throw new Error('Invalid email format');
    }

    if (!request.password || request.password.length < 8) {
      throw new Error('Password must be at least 8 characters long');
    }
  }
}