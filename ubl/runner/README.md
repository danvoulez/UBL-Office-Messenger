# UBL Runner (LAB 512)

**Pull-only job executor for UBL system**

## Architecture — ADR-UBL-Console-001 v1.1

```
┌─────────────────────────────────────────────────────────────┐
│                    UBL Server (LAB 256)                     │
│                                                             │
│  GET /v1/query/commands?pending=1                           │
│  POST /v1/exec.finish                                       │
└─────────────────────────────────────────────────────────────┘
          ▲                              │
          │ poll                         │ receipt
          │                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Runner (LAB 512)                         │
│                    This machine                             │
│                                                             │
│  ┌─────────────────┐    ┌─────────────────┐                │
│  │   pull_only.ts  │───▶│  sandbox-exec   │                │
│  │   (loop)        │    │  (isolation)    │                │
│  └─────────────────┘    └─────────────────┘                │
│           │                     │                           │
│           ▼                     ▼                           │
│  ┌─────────────────┐    ┌─────────────────┐                │
│  │   Allowlist     │    │   Executors     │                │
│  │   (jobs.allow)  │    │   (bash/wasm)   │                │
│  └─────────────────┘    └─────────────────┘                │
└─────────────────────────────────────────────────────────────┘
```

## Key Principles

1. **NO INBOUND CONNECTIONS**: Runner only pulls from UBL
2. **Allowlist-only**: Only jobs in `jobs.allowlist.T.<TENANT>.json` execute
3. **Sandboxed**: All jobs run via `sandbox-exec` with restricted permissions
4. **Receipts**: Every execution produces a signed Receipt sent to UBL

## Usage

```bash
# Set environment
export UBL_URL=http://lab256.local:8080
export TENANT_ID=T.UBL
export RUNNER_TARGET=LAB_512

# Start runner
npm start
```

## Files

| File | Purpose |
|------|---------|
| `pull_only.ts` | Main pull loop |
| `sandbox.sb` | macOS sandbox profile |
| `executors/*.sh` | Job-specific executors |
| `package.json` | Dependencies |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `UBL_URL` | `http://lab256.local:8080` | UBL server address |
| `TENANT_ID` | `T.UBL` | Tenant ID for this runner |
| `RUNNER_TARGET` | `LAB_512` | Target identifier |
| `POLL_INTERVAL` | `5000` | Poll interval in ms |
| `WORK_DIR` | `/tmp/runner-work` | Job work directory |
| `ALLOWLIST_PATH` | `../config/jobs.allowlist.T.UBL.json` | Allowlist file |
| `SANDBOX_PROFILE` | `./sandbox.sb` | Sandbox profile |

## Adding New Job Types

1. Add entry to `config/jobs.allowlist.T.<TENANT>.json`:

```json
{
  "jobType": "my.new.job",
  "risk": "L2",
  "ttl_ms": 120000,
  "requires_step_up": false,
  "fs_scope": "project",
  "network_scope": []
}
```

2. Create executor in `executors/`:

```bash
# executors/my_new_job.sh
#!/bin/bash
PARAMS_FILE="$1"
# ... job logic
```

3. Make executable:

```bash
chmod +x executors/my_new_job.sh
```

## Security

- **Network**: Only outbound to UBL and whitelisted services
- **Filesystem**: Scoped to job work directory
- **No SSH/GPG**: Keys are inaccessible
- **No user data**: Documents/Desktop/Downloads blocked

## Proof of Done

- [ ] Runner starts and logs "pull loop ativo"
- [ ] Polls UBL without errors
- [ ] Executes allowed job type
- [ ] Sends Receipt to UBL (HTTP 200)



