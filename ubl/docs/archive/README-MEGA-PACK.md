# UBL 2.0 — MEGA PACK

This bundle merges:
- ubl2-blueprint-starter-reviewed.zip
- ubl-headless-orchestrator-v4.zip
- ubl-observability-orchestrator-plus-repo.zip
- ubl-id-metrics-ledger-jwt-pack.zip

## Merge Strategy
Overlay (last wins). Newer packs can overwrite files from earlier packs where paths match.

## Quick Start
1) Install Rust toolchain + Node 20 LTS.
2) Postgres + MinIO running (see templates/minio/alias-setup.sh).
3) Apply SQL migrations in `kernel/rust/ubl-server/sql` (including 011_api_tokens.sql).
4) Add Cargo dependencies shown in `kernel/rust/ubl-server/Cargo.additions.toml` to your Cargo.toml.
5) Wire the server following `kernel/rust/ubl-server/src/WIRING_NOTES.md`.
6) Build:
   ```bash
   (cd kernel/rust/ubl-server && cargo build --release)
   (cd clients/ts/sdk && npm i && npm run build)
   (cd clients/cli && npm i && npm run build && npm link)
   ```
7) Test metrics and JWT (see WIRING_NOTES.md).
8) Use CLI to push repo and see ledger tail (SSE).

See `MEGA_MANIFEST.json` for the included packs and order.
