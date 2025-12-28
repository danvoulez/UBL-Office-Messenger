# Contributing to UBL 2.0

## Core Principles

1. **Tests must fail when code is wrong** — No exceptions
2. **No UPDATE/DELETE on ledger** — Append-only, forever
3. **Mind thinks, Body obeys** — Never mix concerns
4. **Specs are frozen** — Changes require new version

---

## Development Setup

### Prerequisites

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable

# Node.js 20+
nvm install 20
nvm use 20

# PostgreSQL 16+
brew install postgresql@16  # macOS
# or use Docker
```

### Build & Test

```bash
# Rust kernel
cd kernel/rust
cargo build --release
cargo test --workspace --no-fail-fast

# TypeScript
cd mind
npm install
npm run build
```

---

## Code Guidelines

### Rust (Body)

- **No `unsafe`** — Ever, without explicit review
- **Error handling** — Use `thiserror` for domain errors
- **Canonical errors** — V1-V8 only (see SPEC-UBL-MEMBRANE)
- **Domain separation** — Hash tags: `ubl:atom\n`, `ubl:link\n`

```rust
// Good
fn hash_atom(bytes: &[u8]) -> String {
    let mut h = Hasher::new();
    h.update(b"ubl:atom\n");
    h.update(bytes);
    hex::encode(h.finalize().as_bytes())
}

// Bad - no domain separation
fn bad_hash(bytes: &[u8]) -> String {
    hex::encode(blake3::hash(bytes).as_bytes())
}
```

### TypeScript (Mind)

- **Semantic only** — No cryptography, no validation
- **Canonical JSON** — Keys sorted, no whitespace
- **Types everywhere** — No `any`

---

## Pull Request Process

1. **Fork** the repository
2. **Create branch** from `main`: `git checkout -b feature/xyz`
3. **Write tests first** — TDD is mandatory
4. **Run all tests**: `cargo test --workspace`
5. **Check formatting**: `cargo fmt --all -- --check`
6. **Submit PR** with clear description

### PR Title Format

```
[AREA] Brief description

Examples:
[kernel] Add rate limiting to WebAuthn
[mind] Fix canonicalization edge case
[docs] Update README with new endpoints
[sql] Add session table migration
```

---

## Architecture Rules

### Never Cross the Boundary

```
Mind (TS)                    Body (Rust)
──────────                   ──────────
✓ Create intent             ✓ Validate signature
✓ Canonicalize JSON         ✓ Check physics
✓ Call HTTP API             ✓ Append to ledger
                            
✗ Sign anything             ✗ Interpret meaning
✗ Validate physics          ✗ Business logic
```

### Commit Structure

```rust
struct LinkCommit {
    // These go in signing_bytes:
    version: u8,
    container_id: String,
    expected_sequence: u64,
    previous_hash: String,
    atom_hash: String,
    intent_class: IntentClass,
    physics_delta: i128,
    
    // These are EXCLUDED from signing_bytes:
    pact: Option<PactProof>,
    author_pubkey: String,
    signature: String,
}
```

---

## Testing Requirements

### Unit Tests

Every module must have tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy_path() { ... }
    
    #[test]
    fn test_error_case() { ... }
    
    #[test]
    fn test_edge_case() { ... }
}
```

### Integration Tests

For HTTP routes:

```bash
# Health check
curl http://localhost:8080/health | jq

# Rate limiting
for i in {1..6}; do
  curl -X POST http://localhost:8080/id/register/begin \
    -H 'Content-Type: application/json' \
    -d '{"username":"test"}'
done
# 5 OK, 1 REJECT (429)
```

### Guardrail Tests

See [HARDENING.md](HARDENING.md) for:
- `signing_bytes_canon.rs` — Prevents signature over wrong fields
- `atom_hash_binding.rs` — Ensures JSON✯Atomic binding
- `physics_invariants.rs` — Enforces thermodynamics

---

## Security

### Reporting Vulnerabilities

Email security issues to the maintainers directly. Do not open public issues.

### Review Checklist

- [ ] No hardcoded secrets
- [ ] Input validation on all endpoints
- [ ] Rate limiting where appropriate
- [ ] CORS configured correctly
- [ ] HttpOnly cookies for sessions
- [ ] SERIALIZABLE transactions for ledger

---

## Documentation

### Where to Document

| Type | Location |
|------|----------|
| API changes | OpenAPI spec + README |
| Architecture | ARCHITECTURE.md |
| Security | HARDENING.md |
| Philosophy | PHILOSOPHY.md |
| Specs | specs/ directory |
| Status | WEBAUTHN_PRODUCTION_STATUS.md |

### Markdown Style

- Use ATX headers (`#`, `##`, `###`)
- Code blocks with language tags
- Tables for structured data
- Links for cross-references

---

## License

By contributing, you agree that your contributions will be licensed under Apache 2.0.
