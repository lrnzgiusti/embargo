/**
 * Cryptographic utility functions
 */
import * as crypto from 'crypto';

// Email validation regex
const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

/**
 * Validate email address format
 */
export function validateEmail(email: string): boolean {
  if (!email || typeof email !== 'string') {
    return false;
  }
  return EMAIL_REGEX.test(email.trim().toLowerCase());
}

/**
 * Hash a password using bcrypt-like algorithm simulation
 */
export function hashPassword(password: string, salt?: string): string {
  if (!password) {
    throw new Error('Password cannot be empty');
  }

  const saltValue = salt || generateSalt();
  const hash = crypto.pbkdf2Sync(password, saltValue, 10000, 64, 'sha256');
  
  return `${saltValue}:${hash.toString('hex')}`;
}

/**
 * Verify a password against its hash
 */
export function verifyPassword(password: string, hash: string): boolean {
  try {
    const [salt, originalHash] = hash.split(':');
    const verifyHash = crypto.pbkdf2Sync(password, salt, 10000, 64, 'sha256');
    
    return originalHash === verifyHash.toString('hex');
  } catch (error) {
    return false;
  }
}

/**
 * Generate a random salt
 */
export function generateSalt(length: number = 32): string {
  return crypto.randomBytes(length).toString('hex');
}

/**
 * Generate a random token
 */
export function generateToken(length: number = 32): string {
  return crypto.randomBytes(length).toString('hex');
}

/**
 * Generate a UUID v4
 */
export function generateUUID(): string {
  return crypto.randomUUID();
}

/**
 * Create an MD5 hash of the input
 */
export function md5Hash(input: string): string {
  return crypto.createHash('md5').update(input).digest('hex');
}

/**
 * Create a SHA-256 hash of the input
 */
export function sha256Hash(input: string): string {
  return crypto.createHash('sha256').update(input).digest('hex');
}

/**
 * Create an HMAC signature
 */
export function createHMAC(data: string, secret: string, algorithm: string = 'sha256'): string {
  return crypto.createHmac(algorithm, secret).update(data).digest('hex');
}

/**
 * Verify an HMAC signature
 */
export function verifyHMAC(data: string, signature: string, secret: string, algorithm: string = 'sha256'): boolean {
  const expectedSignature = createHMAC(data, secret, algorithm);
  return crypto.timingSafeEqual(Buffer.from(signature), Buffer.from(expectedSignature));
}

/**
 * Encrypt data using AES-256-GCM
 */
export function encrypt(data: string, key: string): EncryptedData {
  const iv = crypto.randomBytes(16);
  const cipher = crypto.createCipher('aes-256-gcm', key);
  
  let encrypted = cipher.update(data, 'utf8', 'hex');
  encrypted += cipher.final('hex');
  
  const authTag = cipher.getAuthTag();
  
  return {
    encrypted,
    iv: iv.toString('hex'),
    authTag: authTag.toString('hex'),
  };
}

/**
 * Decrypt data using AES-256-GCM
 */
export function decrypt(encryptedData: EncryptedData, key: string): string {
  const decipher = crypto.createDecipher('aes-256-gcm', key);
  
  decipher.setAuthTag(Buffer.from(encryptedData.authTag, 'hex'));
  
  let decrypted = decipher.update(encryptedData.encrypted, 'hex', 'utf8');
  decrypted += decipher.final('utf8');
  
  return decrypted;
}

/**
 * Generate a cryptographically secure random number
 */
export function secureRandom(min: number = 0, max: number = 1): number {
  const range = max - min;
  const bytesNeeded = Math.ceil(Math.log2(range) / 8);
  const maxValue = Math.pow(256, bytesNeeded) - 1;
  const randomValue = crypto.randomBytes(bytesNeeded).readUIntBE(0, bytesNeeded);
  
  return Math.floor((randomValue / maxValue) * range) + min;
}

/**
 * Generate a secure random string
 */
export function secureRandomString(length: number, charset?: string): string {
  const defaultCharset = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  const chars = charset || defaultCharset;
  
  let result = '';
  for (let i = 0; i < length; i++) {
    const randomIndex = secureRandom(0, chars.length);
    result += chars[randomIndex];
  }
  
  return result;
}

/**
 * Derive a key from a password using PBKDF2
 */
export function deriveKey(password: string, salt: string, iterations: number = 100000, keyLength: number = 32): Buffer {
  return crypto.pbkdf2Sync(password, salt, iterations, keyLength, 'sha256');
}

/**
 * Create a digital signature using RSA
 */
export function createSignature(data: string, privateKey: string): string {
  const sign = crypto.createSign('RSA-SHA256');
  sign.update(data);
  return sign.sign(privateKey, 'hex');
}

/**
 * Verify a digital signature using RSA
 */
export function verifySignature(data: string, signature: string, publicKey: string): boolean {
  const verify = crypto.createVerify('RSA-SHA256');
  verify.update(data);
  return verify.verify(publicKey, signature, 'hex');
}

// Types and interfaces
export interface EncryptedData {\n  encrypted: string;\n  iv: string;\n  authTag: string;\n}\n\nexport interface KeyPair {\n  publicKey: string;\n  privateKey: string;\n}\n\n/**\n * Generate an RSA key pair\n */\nexport function generateKeyPair(keySize: number = 2048): KeyPair {\n  const { publicKey, privateKey } = crypto.generateKeyPairSync('rsa', {\n    modulusLength: keySize,\n    publicKeyEncoding: {\n      type: 'spki',\n      format: 'pem',\n    },\n    privateKeyEncoding: {\n      type: 'pkcs8',\n      format: 'pem',\n    },\n  });\n  \n  return { publicKey, privateKey };\n}\n\n/**\n * Constant-time string comparison to prevent timing attacks\n */\nexport function constantTimeCompare(a: string, b: string): boolean {\n  if (a.length !== b.length) {\n    return false;\n  }\n  \n  return crypto.timingSafeEqual(Buffer.from(a), Buffer.from(b));\n}