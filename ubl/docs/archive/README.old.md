# 🚀 UBL 2.0 - Universal Boundary Ledger

> **The Mind thinks. The Body obeys laws.**

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    MIND (TypeScript)                         │
│                                                              │
│  packages/ubl-cortex                                        │
│  ├── Semantic intents (meaning)                             │
│  ├── Canonicalization (Intent → Atom)                       │
│  └── Orchestration (prepare → commit)                       │
│                                                              │
│  "Thinks but has no physical authority"                     │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ HTTP POST /commit
                         │ LinkCommit { atom_hash, physics_delta, ... }
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                    BODY (Rust)                               │
│                                                              │
│  crates/                                                    │
│  ├── ubl-atom      Canonical JSON serialization             │
│  ├── ubl-link      The commit envelope                      │
│  ├── ubl-kernel    BLAKE3 + Ed25519 crypto                  │
│  ├── ubl-ledger    Append-only hash chain                   │
│  ├── ubl-membrane  Validation (< 1ms)                       │
│  └── ubl-server    HTTP interface                           │
│                                                              │
│  "Doesn't think, only obeys physical laws"                  │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

### 1. Start the Body (Rust)

```bash
cd "/Users/voulezvous/UBL 2.0"

# Build and run
cargo run -p ubl-server

# Or with custom container
CONTAINER_ID=wallet_alice PORT=3000 cargo run -p ubl-server
```

### 2. Start the Mind (TypeScript)

```bash
cd "/Users/voulezvous/UBL 2.0/packages/ubl-cortex"

# Install dependencies
npm install

# Run example
npx tsx src/index.ts
```

## API Endpoints

### `GET /health`
Health check.

### `GET /state`
Get current ledger state.

```json
{
  "container_id": "default",
  "sequence": 5,
  "last_hash": "abc123...",
  "physical_balance": 900,
  "merkle_root": "def456..."
}
```

### `POST /commit`
Submit a commit to the ledger.

**Request:**
```json
{
  "version": 1,
  "container_id": "default",
  "expected_sequence": 6,
  "previous_hash": "abc123...",
  "atom_hash": "semantic_hash...",
  "intent_class": "conservation",
  "physics_delta": -100,
  "author_pubkey": "ed25519_pubkey_hex",
  "signature": "ed25519_signature_hex"
}
```

**Response (Success):**
```json
{
  "status": "ACCEPTED",
  "receipt": {
    "entry_hash": "xyz789...",
    "sequence": 6,
    "timestamp": 1735142400,
    "container_id": "default"
  }
}
```

**Response (Failure):**
```json
{
  "status": "REJECTED",
  "error": "V7: Physics violation - conservation requires balance >= 0",
  "code": "V7_CONSERVATION_VIOLATION"
}
```

## Intent Classes

| Class | Delta Rule | Use Case |
|-------|------------|----------|
| `observation` | Δ = 0 | Read-only operations |
| `conservation` | balance >= 0 | Move value (transfer) |
| `entropy` | any | Create/destroy value |
| `evolution` | any | Change rules |

## Validation Rules (Membrane)

| Code | Description |
|------|-------------|
| V1 | Version must be 1 |
| V2 | Container ID must match |
| V3 | Signature must verify |
| V4 | Previous hash must match (causality) |
| V5 | Sequence must be next |
| V6 | Atom hash must be valid |
| V7 | Physics laws must hold |

## SPEC References

- **SPEC-UBL-CORE v1.0** - Container model: C := ⟨id, L, S, H, Φ⟩
- **SPEC-UBL-ATOM v1.0** - Canonical JSON serialization
- **SPEC-UBL-LINK v1.0** - The commit envelope
- **SPEC-UBL-MEMBRANE v1.0** - Validation rules
- **SPEC-UBL-LEDGER v1.0** - Append-only history

## Development

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p ubl-membrane

# Build release
cargo build --release
```

## License

MIT
