/**
 * Runner Pull-Only Loop — ADR-UBL-Console-001 v1.1
 * 
 * This is the execution agent for LAB 512.
 * NO INBOUND CONNECTIONS. Only pulls from UBL.
 * 
 * Flow:
 * 1. Poll GET /v1/query/commands?pending=1
 * 2. Execute job in sandbox
 * 3. POST /v1/exec.finish with Receipt
 */

import { spawn } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import * as crypto from 'crypto';

// ============================================
// CONFIGURATION
// ============================================

const CONFIG = {
  UBL_URL: process.env.UBL_URL || 'http://lab256.local:8080',
  TENANT_ID: process.env.TENANT_ID || 'T.UBL',
  TARGET: process.env.RUNNER_TARGET || 'LAB_512',
  POLL_INTERVAL_MS: parseInt(process.env.POLL_INTERVAL || '5000'),
  SANDBOX_PROFILE: process.env.SANDBOX_PROFILE || './sandbox.sb',
  WORK_DIR: process.env.WORK_DIR || '/tmp/runner-work',
  ALLOWLIST_PATH: process.env.ALLOWLIST_PATH || '../config/jobs.allowlist.T.UBL.json',
};

// ============================================
// TYPES
// ============================================

interface Command {
  jti: string;
  tenant_id: string;
  job_id: string;
  job_type: string;
  params: Record<string, any>;
  subject_hash: string;
  policy_hash: string;
  permit: Record<string, any>;
  target: string;
  office_id: string;
  pending: number;
  issued_at: number;
}

interface Receipt {
  tenant_id: string;
  jobId: string;
  status: 'success' | 'error';
  finished_at: number;
  logs_hash: string;
  artifacts: string[];
  usage: Record<string, any>;
  error?: string;
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

// ============================================
// ALLOWLIST
// ============================================

let allowlist: Allowlist | null = null;

function loadAllowlist(): Allowlist {
  if (allowlist) return allowlist;
  
  const raw = fs.readFileSync(CONFIG.ALLOWLIST_PATH, 'utf-8');
  allowlist = JSON.parse(raw);
  console.log(`[Runner] Loaded allowlist: ${allowlist!.jobs.length} job types`);
  return allowlist!;
}

function isJobAllowed(jobType: string): AllowlistEntry | null {
  const list = loadAllowlist();
  
  // Check denied patterns first
  for (const pattern of list.denied_patterns) {
    const regex = new RegExp('^' + pattern.replace(/\*/g, '.*') + '$');
    if (regex.test(jobType)) {
      console.log(`[Runner] Job ${jobType} matches denied pattern ${pattern}`);
      return null;
    }
  }
  
  // Find in allowlist
  return list.jobs.find(j => j.jobType === jobType) || null;
}

// ============================================
// HASH UTILITIES
// ============================================

function blake3Hex(data: string): string {
  // Note: In production, use actual blake3 library
  // For now, using SHA256 as fallback (should be replaced)
  const hash = crypto.createHash('sha256').update(data).digest('hex');
  return `blake3:${hash.substring(0, 64)}`;
}

// ============================================
// UBL CLIENT
// ============================================

async function queryPendingCommands(): Promise<Command[]> {
  const url = new URL('/v1/query/commands', CONFIG.UBL_URL);
  url.searchParams.set('tenant_id', CONFIG.TENANT_ID);
  url.searchParams.set('target', CONFIG.TARGET);
  url.searchParams.set('pending', '1');
  url.searchParams.set('limit', '1');
  
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

async function submitReceipt(receipt: Receipt): Promise<boolean> {
  const url = new URL('/v1/exec.finish', CONFIG.UBL_URL);
  
  try {
    const res = await fetch(url.toString(), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(receipt),
    });
    
    if (!res.ok) {
      console.error(`[Runner] Receipt submit failed: ${res.status}`);
      return false;
    }
    
    console.log(`[Runner] Receipt submitted for ${receipt.jobId}`);
    return true;
  } catch (e) {
    console.error(`[Runner] Receipt error:`, e);
    return false;
  }
}

// ============================================
// SANDBOX EXECUTION
// ============================================

async function executeInSandbox(
  cmd: Command,
  entry: AllowlistEntry
): Promise<{ success: boolean; logs: string; artifacts: string[] }> {
  const jobDir = path.join(CONFIG.WORK_DIR, cmd.job_id);
  fs.mkdirSync(jobDir, { recursive: true });
  
  // Write job params for the executor
  const paramsFile = path.join(jobDir, 'params.json');
  fs.writeFileSync(paramsFile, JSON.stringify(cmd.params, null, 2));
  
  // Build sandbox command
  // macOS: sandbox-exec -f <profile> <command>
  const executorScript = path.join(__dirname, 'executors', `${cmd.job_type.replace(/\./g, '_')}.sh`);
  
  if (!fs.existsSync(executorScript)) {
    return {
      success: false,
      logs: `No executor found for job type: ${cmd.job_type}`,
      artifacts: [],
    };
  }
  
  return new Promise((resolve) => {
    const logs: string[] = [];
    const artifacts: string[] = [];
    
    const proc = spawn('sandbox-exec', [
      '-f', CONFIG.SANDBOX_PROFILE,
      'bash', executorScript, paramsFile,
    ], {
      cwd: jobDir,
      env: {
        ...process.env,
        JOB_ID: cmd.job_id,
        JOB_TYPE: cmd.job_type,
        TENANT_ID: cmd.tenant_id,
        FS_SCOPE: entry.fs_scope,
      },
    });
    
    proc.stdout.on('data', (data) => {
      const line = data.toString();
      logs.push(line);
      console.log(`[Job ${cmd.job_id}] ${line.trim()}`);
    });
    
    proc.stderr.on('data', (data) => {
      const line = data.toString();
      logs.push(`[stderr] ${line}`);
      console.error(`[Job ${cmd.job_id}] ${line.trim()}`);
    });
    
    proc.on('close', (code) => {
      // Collect artifacts
      const artifactsDir = path.join(jobDir, 'artifacts');
      if (fs.existsSync(artifactsDir)) {
        const files = fs.readdirSync(artifactsDir);
        artifacts.push(...files.map(f => path.join(artifactsDir, f)));
      }
      
      resolve({
        success: code === 0,
        logs: logs.join(''),
        artifacts,
      });
    });
    
    proc.on('error', (err) => {
      resolve({
        success: false,
        logs: `Process error: ${err.message}`,
        artifacts: [],
      });
    });
  });
}

// ============================================
// MAIN LOOP
// ============================================

async function processCommand(cmd: Command): Promise<void> {
  console.log(`[Runner] Processing job ${cmd.job_id} (${cmd.job_type})`);
  
  // Check allowlist
  const entry = isJobAllowed(cmd.job_type);
  if (!entry) {
    const receipt: Receipt = {
      tenant_id: cmd.tenant_id,
      jobId: cmd.job_id,
      status: 'error',
      finished_at: Date.now(),
      logs_hash: blake3Hex('Job type not in allowlist'),
      artifacts: [],
      usage: {},
      error: `Job type ${cmd.job_type} not in allowlist`,
    };
    await submitReceipt(receipt);
    return;
  }
  
  // Execute in sandbox
  const startTime = Date.now();
  const result = await executeInSandbox(cmd, entry);
  const duration = Date.now() - startTime;
  
  // Build receipt
  const receipt: Receipt = {
    tenant_id: cmd.tenant_id,
    jobId: cmd.job_id,
    status: result.success ? 'success' : 'error',
    finished_at: Date.now(),
    logs_hash: blake3Hex(result.logs),
    artifacts: result.artifacts,
    usage: {
      duration_ms: duration,
      fs_scope: entry.fs_scope,
    },
    error: result.success ? undefined : result.logs.substring(0, 1000),
  };
  
  await submitReceipt(receipt);
}

async function pullLoop(): Promise<void> {
  console.log(`[Runner] Starting pull-only loop`);
  console.log(`  UBL: ${CONFIG.UBL_URL}`);
  console.log(`  Tenant: ${CONFIG.TENANT_ID}`);
  console.log(`  Target: ${CONFIG.TARGET}`);
  console.log(`  Poll interval: ${CONFIG.POLL_INTERVAL_MS}ms`);
  
  // Ensure work directory exists
  fs.mkdirSync(CONFIG.WORK_DIR, { recursive: true });
  
  // Load allowlist
  try {
    loadAllowlist();
  } catch (e) {
    console.error(`[Runner] Failed to load allowlist:`, e);
    process.exit(1);
  }
  
  while (true) {
    try {
      const commands = await queryPendingCommands();
      
      if (commands.length === 0) {
        // No pending commands
      } else {
        for (const cmd of commands) {
          await processCommand(cmd);
        }
      }
    } catch (e) {
      console.error(`[Runner] Loop error:`, e);
    }
    
    // Wait before next poll
    await new Promise(resolve => setTimeout(resolve, CONFIG.POLL_INTERVAL_MS));
  }
}

// ============================================
// ENTRY POINT
// ============================================

pullLoop().catch(console.error);



