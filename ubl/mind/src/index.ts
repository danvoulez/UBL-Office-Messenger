/**
 * UBL Cortex - The Mind
 * 
 * This is where semantics live. The Mind thinks but has no physical authority.
 * All effects must be materialized through the Body (Rust) via ubl-link commits.
 * 
 * SPEC-UBL-CORTEX
 */

import { createHash } from 'crypto';

// =============================================================================
// TYPES - The Language (L)
// =============================================================================

/** Intent classes matching SPEC-UBL-LINK */
export type IntentClass = 'observation' | 'conservation' | 'entropy' | 'evolution';

/** A semantic intent (the meaning, not the physics) */
export interface Intent {
  readonly type: string;
  readonly payload: Record<string, unknown>;
  readonly reason?: string;
}

/** The link commit structure (matches Rust) */
export interface LinkCommit {
  version: number;
  container_id: string;
  expected_sequence: number;
  previous_hash: string;
  atom_hash: string;
  intent_class: IntentClass;
  physics_delta: number;
  author_pubkey: string;
  signature: string;
}

/** Ledger state from the Body */
export interface LedgerState {
  container_id: string;
  sequence: number;
  last_hash: string;
  physical_balance: number;
  merkle_root: string;
}

/** Receipt from a successful commit */
export interface LinkReceipt {
  entry_hash: string;
  sequence: number;
  timestamp: number;
  container_id: string;
}

/** Response from commit endpoint */
export type CommitResponse =
  | { status: 'ACCEPTED'; receipt: LinkReceipt }
  | { status: 'REJECTED'; error: string; code: string };

// =============================================================================
// UBL-ATOM - Canonicalization
// =============================================================================

/**
 * Canonicalize JSON for deterministic hashing
 * Keys are sorted alphabetically, recursively
 */
export function canonicalize(data: unknown): string {
  return JSON.stringify(sortKeys(data));
}

function sortKeys(obj: unknown): unknown {
  if (obj === null || typeof obj !== 'object') {
    return obj;
  }
  
  if (Array.isArray(obj)) {
    return obj.map(sortKeys);
  }
  
  const sorted: Record<string, unknown> = {};
  const keys = Object.keys(obj as Record<string, unknown>).sort();
  
  for (const key of keys) {
    sorted[key] = sortKeys((obj as Record<string, unknown>)[key]);
  }
  
  return sorted;
}

/**
 * Hash an atom using SHA-256 (would use BLAKE3 in production via WASM)
 */
export function hashAtom(canonical: string): string {
  return createHash('sha256').update(canonical).digest('hex');
}

// =============================================================================
// UBL-CORTEX - The Orchestrator
// =============================================================================

export interface CortexConfig {
  /** URL of the UBL Body (Rust server) */
  bodyUrl: string;
  /** Container ID to operate on */
  containerId: string;
  /** Author's public key (hex) */
  authorPubkey: string;
  /** Signing function (would use Ed25519 in production) */
  sign?: (message: Uint8Array) => string;
}

/**
 * The Cortex - Orchestrator between Mind and Body
 */
export class Cortex {
  private readonly config: CortexConfig;

  constructor(config: CortexConfig) {
    this.config = config;
  }

  /**
   * Get current state from the Body
   */
  async getState(): Promise<LedgerState> {
    const response = await fetch(`${this.config.bodyUrl}/state`);
    if (!response.ok) {
      throw new Error(`Failed to get state: ${response.statusText}`);
    }
    return response.json();
  }

  /**
   * Transform a semantic intent into a physical commit
   */
  async prepareCommit(
    intent: Intent,
    intentClass: IntentClass,
    physicsDelta: number
  ): Promise<LinkCommit> {
    // 1. Get current state
    const state = await this.getState();

    // 2. Canonicalize intent and hash
    const canonical = canonicalize(intent);
    const atomHash = hashAtom(canonical);

    // 3. Build the commit
    const commit: LinkCommit = {
      version: 1,
      container_id: this.config.containerId,
      expected_sequence: state.sequence + 1,
      previous_hash: state.last_hash,
      atom_hash: atomHash,
      intent_class: intentClass,
      physics_delta: physicsDelta,
      author_pubkey: this.config.authorPubkey,
      signature: 'mock', // Would sign in production
    };

    return commit;
  }

  /**
   * Submit a commit to the Body
   */
  async commit(link: LinkCommit): Promise<CommitResponse> {
    const response = await fetch(`${this.config.bodyUrl}/commit`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(link),
    });

    return response.json();
  }

  /**
   * Execute an intent: prepare and commit in one call
   */
  async execute(
    intent: Intent,
    intentClass: IntentClass,
    physicsDelta: number
  ): Promise<CommitResponse> {
    const link = await this.prepareCommit(intent, intentClass, physicsDelta);
    return this.commit(link);
  }

  /**
   * Observe (read-only, delta = 0)
   */
  async observe(intent: Intent): Promise<CommitResponse> {
    return this.execute(intent, 'observation', 0);
  }

  /**
   * Conserve (move value, balance must remain >= 0)
   */
  async conserve(intent: Intent, delta: number): Promise<CommitResponse> {
    return this.execute(intent, 'conservation', delta);
  }

  /**
   * Create/destroy value (entropy)
   */
  async entropy(intent: Intent, delta: number): Promise<CommitResponse> {
    return this.execute(intent, 'entropy', delta);
  }
}

// =============================================================================
// EXAMPLE USAGE
// =============================================================================

async function main() {
  console.log('🧠 UBL Cortex v2.0');
  console.log('   The Mind that thinks but has no physical authority\n');

  const cortex = new Cortex({
    bodyUrl: 'http://localhost:3000',
    containerId: 'default',
    authorPubkey: 'mock_pubkey',
  });

  try {
    // 1. Check current state
    console.log('📊 Getting current state...');
    const state = await cortex.getState();
    console.log(`   Container: ${state.container_id}`);
    console.log(`   Sequence: ${state.sequence}`);
    console.log(`   Balance: ${state.physical_balance}`);
    console.log(`   Last hash: ${state.last_hash.slice(0, 16)}...`);
    console.log();

    // 2. Create some value (entropy)
    console.log('💰 Creating 1000 credits (entropy)...');
    const createIntent: Intent = {
      type: 'credits:create',
      payload: { amount: 1000 },
      reason: 'Initial funding',
    };
    
    const createResult = await cortex.entropy(createIntent, 1000);
    if (createResult.status === 'ACCEPTED') {
      console.log(`   ✅ ACCEPTED - Receipt: ${createResult.receipt.entry_hash.slice(0, 16)}...`);
    } else {
      console.log(`   ⛔ REJECTED - ${createResult.error}`);
    }
    console.log();

    // 3. Transfer some value (conservation)
    console.log('📤 Transferring 100 credits (conservation)...');
    const transferIntent: Intent = {
      type: 'credits:transfer',
      payload: { to: 'bob', amount: 100 },
      reason: 'Payment for services',
    };
    
    const transferResult = await cortex.conserve(transferIntent, -100);
    if (transferResult.status === 'ACCEPTED') {
      console.log(`   ✅ ACCEPTED - Receipt: ${transferResult.receipt.entry_hash.slice(0, 16)}...`);
    } else {
      console.log(`   ⛔ REJECTED - ${transferResult.error}`);
    }
    console.log();

    // 4. Final state
    console.log('📊 Final state...');
    const finalState = await cortex.getState();
    console.log(`   Sequence: ${finalState.sequence}`);
    console.log(`   Balance: ${finalState.physical_balance}`);

  } catch (error) {
    console.error('❌ Error:', error);
    console.log('\n💡 Make sure the UBL Body (Rust server) is running:');
    console.log('   cd /Users/voulezvous/UBL\\ 2.0 && cargo run -p ubl-server');
  }
}

// Run if called directly
const isMainModule = import.meta.url === `file://${process.argv[1]}`;
if (isMainModule) {
  main();
}
