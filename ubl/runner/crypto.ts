/**
 * Runner Cryptographic Utilities
 * 
 * Ed25519 signing for receipts, canonical JSON, BLAKE3 hashing.
 */

import * as ed from '@noble/ed25519';
import { sha512 } from '@noble/hashes/sha512';
import { blake3 } from '@noble/hashes/blake3';
import * as fs from 'fs';

// Required for @noble/ed25519
ed.etc.sha512Sync = (...m) => sha512(ed.etc.concatBytes(...m));

// =============================================================================
// CANONICAL JSON (matches Rust ubl_atom)
// =============================================================================

export function canonicalize(value: unknown): Uint8Array {
  const json = canonicalJson(value);
  return new TextEncoder().encode(json);
}

function canonicalJson(value: unknown): string {
  if (value === null) return 'null';
  if (typeof value === 'boolean') return value.toString();
  if (typeof value === 'number') {
    if (!Number.isFinite(value)) throw new Error('Non-finite number');
    return value.toString();
  }
  if (typeof value === 'string') return JSON.stringify(value);
  if (Array.isArray(value)) {
    const items = value.map(canonicalJson);
    return `[${items.join(',')}]`;
  }
  if (typeof value === 'object') {
    const keys = Object.keys(value as Record<string, unknown>).sort();
    const pairs = keys.map(k => {
      const v = (value as Record<string, unknown>)[k];
      return `${JSON.stringify(k)}:${canonicalJson(v)}`;
    });
    return `{${pairs.join(',')}}`;
  }
  throw new Error(`Cannot canonicalize type: ${typeof value}`);
}

// =============================================================================
// BLAKE3 HASHING
// =============================================================================

export function blake3Hex(data: Uint8Array | string): string {
  const bytes = typeof data === 'string' ? new TextEncoder().encode(data) : data;
  const hash = blake3(bytes);
  return `blake3:${Buffer.from(hash).toString('hex')}`;
}

// =============================================================================
// ED25519 RUNNER KEY
// =============================================================================

let runnerPrivateKey: Uint8Array | null = null;
let runnerPublicKey: Uint8Array | null = null;

/**
 * Load runner private key from file or environment.
 * Key should be 32 bytes (seed) or 64 bytes (full key) hex-encoded.
 */
export function loadRunnerKey(): void {
  // Try environment variable first
  const keyHex = process.env.RUNNER_PRIVATE_KEY;
  if (keyHex) {
    const keyBytes = Buffer.from(keyHex, 'hex');
    if (keyBytes.length === 32) {
      runnerPrivateKey = keyBytes;
      runnerPublicKey = ed.getPublicKey(runnerPrivateKey);
      console.log('[Crypto] Loaded runner key from environment');
      return;
    }
  }

  // Try file
  const keyPath = process.env.RUNNER_KEY_PATH || './runner.key';
  if (fs.existsSync(keyPath)) {
    const keyHexFile = fs.readFileSync(keyPath, 'utf-8').trim();
    const keyBytes = Buffer.from(keyHexFile, 'hex');
    if (keyBytes.length === 32) {
      runnerPrivateKey = keyBytes;
      runnerPublicKey = ed.getPublicKey(runnerPrivateKey);
      console.log(`[Crypto] Loaded runner key from ${keyPath}`);
      return;
    }
  }

  // Generate ephemeral key for development
  console.warn('[Crypto] ⚠️  No runner key found, generating ephemeral key (DEV ONLY)');
  runnerPrivateKey = ed.utils.randomPrivateKey();
  runnerPublicKey = ed.getPublicKey(runnerPrivateKey);
  
  // Log public key for registration
  console.log(`[Crypto] Runner public key (register in UBL): ${Buffer.from(runnerPublicKey).toString('hex')}`);
}

/**
 * Get runner public key (hex)
 */
export function getRunnerPublicKeyHex(): string {
  if (!runnerPublicKey) loadRunnerKey();
  return Buffer.from(runnerPublicKey!).toString('hex');
}

/**
 * Sign data with runner private key.
 * Returns: "ed25519:<base64url_signature>"
 */
export function signWithRunnerKey(data: Uint8Array): string {
  if (!runnerPrivateKey) loadRunnerKey();
  const signature = ed.sign(data, runnerPrivateKey!);
  const b64 = Buffer.from(signature).toString('base64url');
  return `ed25519:${b64}`;
}

/**
 * Verify a signature (for testing)
 */
export function verifySignature(data: Uint8Array, sigTagged: string, pubkeyHex: string): boolean {
  if (!sigTagged.startsWith('ed25519:')) return false;
  const sigB64 = sigTagged.replace('ed25519:', '');
  const sigBytes = Buffer.from(sigB64, 'base64url');
  const pubkeyBytes = Buffer.from(pubkeyHex, 'hex');
  return ed.verify(sigBytes, data, pubkeyBytes);
}

// =============================================================================
// RECEIPT SIGNING
// =============================================================================

export interface ReceiptPayload {
  command_id: string;
  permit_jti: string;
  binding_hash: string;
  runner_id: string;
  status: 'OK' | 'ERROR';
  logs_hash: string;
  ret: unknown;
}

/**
 * Build and sign a receipt.
 * Returns the receipt with sig_runner field.
 */
export function signReceipt(payload: ReceiptPayload): ReceiptPayload & { sig_runner: string } {
  // Canonicalize payload
  const canonical = canonicalize(payload);
  
  // Sign
  const sig = signWithRunnerKey(canonical);
  
  return {
    ...payload,
    sig_runner: sig,
  };
}

// =============================================================================
// KEY GENERATION UTILITY
// =============================================================================

/**
 * Generate a new runner keypair and save to file.
 * Use this to create keys for production runners.
 */
export function generateRunnerKeypair(outputPath: string): { privateKeyHex: string; publicKeyHex: string } {
  const privateKey = ed.utils.randomPrivateKey();
  const publicKey = ed.getPublicKey(privateKey);
  
  const privateKeyHex = Buffer.from(privateKey).toString('hex');
  const publicKeyHex = Buffer.from(publicKey).toString('hex');
  
  // Save private key
  fs.writeFileSync(outputPath, privateKeyHex, { mode: 0o600 });
  console.log(`[Crypto] Saved private key to ${outputPath}`);
  console.log(`[Crypto] Public key (register in UBL): ${publicKeyHex}`);
  
  return { privateKeyHex, publicKeyHex };
}

// Initialize on import
loadRunnerKey();

