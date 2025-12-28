/**
 * Runner Pull-Only Loop — Console v1.1 with Ed25519 Signatures
 * 
 * This is the execution agent for LAB 512.
 * NO INBOUND CONNECTIONS. Only pulls from UBL.
 * 
 * Security:
 * - All receipts signed with Ed25519 runner key
 * - Receipts include command_id, permit_jti, binding_hash
 * - UBL verifies signature before accepting
 * 
 * Flow:
 * 1. Poll GET /v1/query/commands?pending=true
 * 2. Execute job in sandbox
 * 3. Sign receipt with Ed25519
 * 4. POST /v1/exec.finish with signed Receipt
 */

import { spawn } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import { 
  loadRunnerKey, 
  getRunnerPublicKeyHex, 
  signReceipt, 
  blake3Hex, 
  canonicalize,
  type ReceiptPayload 
} from './crypto.js';

// =============================================================================
// CONFIGURATION
// =============================================================================

const CONFIG = {
  UBL_URL: process.env.UBL_URL || 'http://lab256.local:8080',
  RUNNER_ID: process.env.RUNNER_ID || 'LAB_512',
  TARGET: process.env.RUNNER_TARGET || 'LAB_512',
  POLL_INTERVAL_MS: parseInt(process.env.POLL_INTERVAL || '5000'),
  SANDBOX_PROFILE: process.env.SANDBOX_PROFILE || './sandbox.sb',
  WORK_DIR: process.env.WORK_DIR || '/tmp/runner-work',
  ALLOWLIST_PATH: process.env.ALLOWLIST_PATH || '../manifests/jobs.allowlist.yaml',
};

// =============================================================================
// TYPES (Console v1.1)
// =============================================================================

interface Command {
  command_id: string;
  permit_jti: string;
  office: string;
  action: string;
  target: string;
  args: Record<string, unknown>;
  risk: string;
  plan_hash: string;
  binding_hash: string;
  pending: boolean;
  created_at_ms: number;
}

interface AllowlistEntry {
  jobType: string;
  risk: string;
  fs_scope: string;
  network_scope: string[];
}

interface Allowlist {
  jobs: AllowlistEntry[];
  denied_patterns: string[];
}

// =============================================================================
// ALLOWLIST
// =============================================================================

let allowlist: Allowlist | null = null;

function loadAllowlist(): Allowlist {
  if (allowlist) return allowlist;
  
  try {
    // Try YAML first
    if (CONFIG.ALLOWLIST_PATH.endsWith('.yaml') || CONFIG.ALLOWLIST_PATH.endsWith('.yml')) {
      // Simple YAML parsing (for basic structure)
      const raw = fs.readFileSync(CONFIG.ALLOWLIST_PATH, 'utf-8');
      // For now, fall back to JSON path
      const jsonPath = CONFIG.ALLOWLIST_PATH.replace(/\.ya?ml$/, '.json');
      if (fs.existsSync(jsonPath)) {
        const jsonRaw = fs.readFileSync(jsonPath, 'utf-8');
        allowlist = JSON.parse(jsonRaw);
      } else {
        // Default minimal allowlist
        allowlist = { jobs: [], denied_patterns: ['*'] };
      }
    } else {
      const raw = fs.readFileSync(CONFIG.ALLOWLIST_PATH, 'utf-8');
      allowlist = JSON.parse(raw);
    }
  } catch (e) {
    console.warn(`[Runner] Failed to load allowlist, using default deny-all:`, e);
    allowlist = { jobs: [], denied_patterns: ['*'] };
  }
  
  console.log(`[Runner] Allowlist: ${allowlist!.jobs.length} job types allowed`);
  return allowlist!;
}

function isJobAllowed(action: string): AllowlistEntry | null {
  const list = loadAllowlist();
  
  // Check denied patterns first
  for (const pattern of list.denied_patterns) {
    const regex = new RegExp('^' + pattern.replace(/\*/g, '.*') + '$');
    if (regex.test(action)) {
      console.log(`[Runner] Action ${action} matches denied pattern ${pattern}`);
      return null;
    }
  }
  
  // Find in allowlist (match by action/jobType)
  return list.jobs.find(j => j.jobType === action) || null;
}

// =============================================================================
// UBL CLIENT
// =============================================================================

async function queryPendingCommands(): Promise<Command[]> {
  const url = new URL('/v1/query/commands', CONFIG.UBL_URL);
  url.searchParams.set('target', CONFIG.TARGET);
  url.searchParams.set('pending', 'true');
  url.searchParams.set('limit', '5');
  
  try {
    const res = await fetch(url.toString());
    if (!res.ok) {
      console.error(`[Runner] Query failed: ${res.status}`);
      return [];
    }
    return await res.json();
  } catch (e) {
    console.error(`[Runner] Query error:`, e);
    return [];
  }
}

async function submitReceipt(receipt: ReceiptPayload & { sig_runner: string }): Promise<boolean> {
  const url = new URL('/v1/exec.finish', CONFIG.UBL_URL);
  
  try {
    const res = await fetch(url.toString(), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(receipt),
    });
    
    if (!res.ok) {
      const body = await res.text();
      console.error(`[Runner] Receipt submit failed: ${res.status} ${body}`);
      return false;
    }
    
    console.log(`[Runner] ✅ Receipt submitted for command ${receipt.command_id}`);
    return true;
  } catch (e) {
    console.error(`[Runner] Receipt error:`, e);
    return false;
  }
}

// =============================================================================
// SANDBOX EXECUTION
// =============================================================================

async function executeInSandbox(
  cmd: Command,
  entry: AllowlistEntry
): Promise<{ success: boolean; logs: string; ret: unknown }> {
  const jobDir = path.join(CONFIG.WORK_DIR, cmd.command_id);
  fs.mkdirSync(jobDir, { recursive: true });
  
  // Write job params for the executor
  const paramsFile = path.join(jobDir, 'params.json');
  fs.writeFileSync(paramsFile, JSON.stringify(cmd.args, null, 2));
  
  // Build executor script path (action name with dots replaced by underscores)
  const executorScript = path.join(
    path.dirname(new URL(import.meta.url).pathname),
    'executors',
    `${cmd.action.replace(/\./g, '_')}.sh`
  );
  
  if (!fs.existsSync(executorScript)) {
    return {
      success: false,
      logs: `No executor found for action: ${cmd.action} (expected: ${executorScript})`,
      ret: null,
    };
  }
  
  return new Promise((resolve) => {
    const logs: string[] = [];
    let ret: unknown = null;
    
    // Detect platform for sandbox
    const isMac = process.platform === 'darwin';
    let proc;
    
    if (isMac && fs.existsSync(CONFIG.SANDBOX_PROFILE)) {
      // macOS sandbox-exec
      proc = spawn('sandbox-exec', [
        '-f', CONFIG.SANDBOX_PROFILE,
        'bash', executorScript, paramsFile,
      ], {
        cwd: jobDir,
        env: {
          ...process.env,
          COMMAND_ID: cmd.command_id,
          ACTION: cmd.action,
          OFFICE: cmd.office,
          TARGET: cmd.target,
          RISK: cmd.risk,
          FS_SCOPE: entry.fs_scope,
          OUTPUT_FILE: path.join(jobDir, 'output.json'),
        },
      });
    } else {
      // No sandbox (development mode)
      console.warn(`[Runner] ⚠️  Running without sandbox (development mode)`);
      proc = spawn('bash', [executorScript, paramsFile], {
        cwd: jobDir,
        env: {
          ...process.env,
          COMMAND_ID: cmd.command_id,
          ACTION: cmd.action,
          OFFICE: cmd.office,
          TARGET: cmd.target,
          RISK: cmd.risk,
          FS_SCOPE: entry.fs_scope,
          OUTPUT_FILE: path.join(jobDir, 'output.json'),
        },
      });
    }
    
    proc.stdout.on('data', (data) => {
      const line = data.toString();
      logs.push(line);
      console.log(`[Job ${cmd.command_id.substring(0, 8)}] ${line.trim()}`);
    });
    
    proc.stderr.on('data', (data) => {
      const line = data.toString();
      logs.push(`[stderr] ${line}`);
      console.error(`[Job ${cmd.command_id.substring(0, 8)}] ${line.trim()}`);
    });
    
    proc.on('close', (code) => {
      // Try to read output.json for return value
      const outputFile = path.join(jobDir, 'output.json');
      if (fs.existsSync(outputFile)) {
        try {
          ret = JSON.parse(fs.readFileSync(outputFile, 'utf-8'));
        } catch (e) {
          logs.push(`[Runner] Failed to parse output.json: ${e}`);
        }
      }
      
      resolve({
        success: code === 0,
        logs: logs.join(''),
        ret,
      });
    });
    
    proc.on('error', (err) => {
      resolve({
        success: false,
        logs: `Process error: ${err.message}`,
        ret: null,
      });
    });
  });
}

// =============================================================================
// MAIN LOOP
// =============================================================================

async function processCommand(cmd: Command): Promise<void> {
  console.log(`[Runner] Processing command ${cmd.command_id.substring(0, 8)}... (${cmd.action})`);
  console.log(`  Office: ${cmd.office}`);
  console.log(`  Target: ${cmd.target}`);
  console.log(`  Risk: ${cmd.risk}`);
  console.log(`  Binding: ${cmd.binding_hash.substring(0, 20)}...`);
  
  const startTime = Date.now();
  let result: { success: boolean; logs: string; ret: unknown };
  
  // Check allowlist
  const entry = isJobAllowed(cmd.action);
  if (!entry) {
    result = {
      success: false,
      logs: `Action ${cmd.action} not in allowlist`,
      ret: { error: 'ACTION_NOT_ALLOWED' },
    };
  } else {
    // Execute in sandbox
    result = await executeInSandbox(cmd, entry);
  }
  
  const duration = Date.now() - startTime;
  
  // Build receipt payload
  const receiptPayload: ReceiptPayload = {
    command_id: cmd.command_id,
    permit_jti: cmd.permit_jti,
    binding_hash: cmd.binding_hash,
    runner_id: CONFIG.RUNNER_ID,
    status: result.success ? 'OK' : 'ERROR',
    logs_hash: blake3Hex(result.logs),
    ret: result.ret ?? { 
      success: result.success, 
      duration_ms: duration,
      error: result.success ? undefined : result.logs.substring(0, 500),
    },
  };
  
  // Sign the receipt
  const signedReceipt = signReceipt(receiptPayload);
  
  console.log(`[Runner] Receipt signed (${signedReceipt.sig_runner.substring(0, 30)}...)`);
  
  // Submit
  await submitReceipt(signedReceipt);
}

async function pullLoop(): Promise<void> {
  console.log('═══════════════════════════════════════════════════════════');
  console.log('  UBL Runner — Pull-Only Executor with Ed25519 Signatures  ');
  console.log('═══════════════════════════════════════════════════════════');
  console.log(`  UBL API:      ${CONFIG.UBL_URL}`);
  console.log(`  Runner ID:    ${CONFIG.RUNNER_ID}`);
  console.log(`  Target:       ${CONFIG.TARGET}`);
  console.log(`  Poll:         ${CONFIG.POLL_INTERVAL_MS}ms`);
  console.log(`  Public Key:   ${getRunnerPublicKeyHex().substring(0, 16)}...`);
  console.log('═══════════════════════════════════════════════════════════');
  
  // Ensure work directory exists
  fs.mkdirSync(CONFIG.WORK_DIR, { recursive: true });
  
  // Load allowlist
  loadAllowlist();
  
  let iteration = 0;
  
  while (true) {
    try {
      const commands = await queryPendingCommands();
      
      if (commands.length > 0) {
        console.log(`[Runner] Found ${commands.length} pending command(s)`);
        for (const cmd of commands) {
          await processCommand(cmd);
        }
      } else if (iteration % 12 === 0) {
        // Log every 60 seconds (12 * 5s) when idle
        console.log(`[Runner] No pending commands, waiting...`);
      }
      
      iteration++;
    } catch (e) {
      console.error(`[Runner] Loop error:`, e);
    }
    
    // Wait before next poll
    await new Promise(resolve => setTimeout(resolve, CONFIG.POLL_INTERVAL_MS));
  }
}

// =============================================================================
// ENTRY POINT
// =============================================================================

console.log('[Runner] Initializing...');
loadRunnerKey();
pullLoop().catch(console.error);
