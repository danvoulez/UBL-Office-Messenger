# UBL Runner v2.0

> Pull-Only Job Executor with Ed25519 Signed Receipts

## 🔐 Security Model

- **Pull-Only**: No inbound connections. Runner only polls UBL for commands.
- **Ed25519 Signatures**: All receipts are signed with the runner's private key.
- **Sandboxed Execution**: Jobs run in macOS sandbox-exec or Linux nsjail.
- **Binding Hash**: Receipts include the permit's binding_hash for audit trail.

## 🚀 Quick Start

### 1. Install Dependencies

```bash
cd ubl/runner
npm install
```

### 2. Generate Runner Keypair

```bash
npm run keygen
# Outputs:
#   Saved private key to ./runner.key
#   Public key (register in UBL): <hex>
```

### 3. Register Runner in UBL

Add the public key to the `ubl_runners` table:

```sql
UPDATE ubl_runners 
SET pubkey_ed25519 = '<your_public_key_hex>'
WHERE runner_id = 'LAB_512';
```

### 4. Start the Runner

```bash
# Development
npm run dev

# Production
RUNNER_PRIVATE_KEY=<hex> npm start
```

## ⚙️ Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `UBL_URL` | `http://lab256.local:8080` | UBL Kernel API URL |
| `RUNNER_ID` | `LAB_512` | Runner identifier |
| `RUNNER_TARGET` | `LAB_512` | Target zone for commands |
| `RUNNER_PRIVATE_KEY` | - | Ed25519 private key (hex) |
| `RUNNER_KEY_PATH` | `./runner.key` | Path to private key file |
| `POLL_INTERVAL` | `5000` | Poll interval in ms |
| `SANDBOX_PROFILE` | `./sandbox.sb` | macOS sandbox profile |
| `WORK_DIR` | `/tmp/runner-work` | Working directory for jobs |

## 📁 Structure

```
runner/
├── pull_only.ts      # Main loop
├── crypto.ts         # Ed25519, BLAKE3, canonical JSON
├── sandbox.sb        # macOS sandbox profile
├── executors/        # Job executor scripts
│   ├── echo_test.sh
│   └── deploy.sh
├── runner.key        # Private key (gitignored!)
└── package.json
```

## 🔧 Creating Executors

Each job type needs an executor script in `executors/`:

```bash
# executors/echo_test.sh
#!/bin/bash
set -e

PARAMS_FILE="$1"
echo "Executing echo_test job"
echo "Params: $(cat "$PARAMS_FILE")"

# Write output
echo '{"result": "success", "message": "Hello from runner"}' > "$OUTPUT_FILE"
```

The script receives:
- `$1`: Path to params.json
- `$OUTPUT_FILE`: Where to write JSON output
- `$COMMAND_ID`, `$ACTION`, `$OFFICE`, `$TARGET`, `$RISK`: Env vars

## 🧪 Testing

```bash
# Test crypto module
npm run test:crypto

# Output:
# Signed: { command_id: 'test', ..., sig_runner: 'ed25519:...' }
# Verify: true
```

## 📜 Receipt Format

```json
{
  "command_id": "abc-123",
  "permit_jti": "def-456",
  "binding_hash": "blake3:...",
  "runner_id": "LAB_512",
  "status": "OK",
  "logs_hash": "blake3:...",
  "ret": { "result": "success" },
  "sig_runner": "ed25519:..."
}
```

The signature is computed over the canonical JSON (sorted keys, no whitespace) of the payload **without** `sig_runner`.

## 🔒 Key Management

**Production:**
- Store private key in secure storage (HSM, Keychain)
- Never commit `runner.key` to git
- Rotate keys periodically
- Use separate keys per environment (LAB_512 vs LAB_256)

**Development:**
- Runner generates ephemeral key on startup if none provided
- Ephemeral public key is logged for temporary registration

## 🌐 Network Zones

```
┌─────────────────────┐     ┌─────────────────────┐
│      LAB 256        │     │      LAB 512        │
│   (UBL Kernel)      │◄────│     (Runner)        │
│                     │     │                     │
│  - API Gateway      │     │  - Pull-Only        │
│  - WebAuthn         │     │  - Sandboxed        │
│  - Ledger           │     │  - Ed25519 Signed   │
└─────────────────────┘     └─────────────────────┘
         ▲
         │ WireGuard
         ▼
┌─────────────────────┐
│      Office         │
│   (LLM Runtime)     │
│                     │
│  - ASC Token        │
│  - No DB Access     │
│  - Constitution     │
└─────────────────────┘
```

---

**UBL World** — *Where execution is proven, not claimed.*
