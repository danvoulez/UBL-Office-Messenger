# C.Office Boundary

The boundary layer for Office is the `ubl_client` module in the Office runtime.

All mutations go through:
- `POST /v1/policy/permit` → Request authorization
- `POST /link/commit` → Commit events to ledger
- `POST /v1/commands/issue` → Queue jobs for Runner

## Signing

Office has its own Ed25519 keypair (loaded via KeyStore).
All commits are signed by the Office's key.

## ASC

Office uses an **Agent Service Credential** (ASC) issued by UBL.
The ASC has limited scopes:
- `containers: ["C.Office"]`
- `intents: ["observation"]` (no entropy/evolution)
- `max_delta: 0`

