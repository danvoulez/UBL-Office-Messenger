# UBL 2.0 — Blueprint Unificado (Repo • Workspace • Deploy + Auth)
**Data:** 2025-12-26  |  **Specs:** UBL v1.0 (FROZEN)

> Repositórios, workspaces e deploys viram containers universais, atravessando a fronteira do real via **TDLN → Membrane → Ledger**.

## Mapa Mental
```
[Container L] --TDLN--> [Membrane/Link + Ledger] --Accept--> [Runner→Receipt]
```
Perfis: **static (Repo)** Δ=0; **live (Workspace/Deploy)** Δ≠0 com PACT/ASC e Runner. Cores: Verde=L256 gateway; Azul=Admin; Preto=L512 runner.

## Identidade (WebAuthn + JWT)
Fluxo: Passkey → sessão (cookie HttpOnly) → **/id/session/token** (JWT Ed25519) → Bearer curto.  
Step-up (admin) com WebAuthn (10min). Rate-limit, TTL de challenge (5min/2min), origin pinado, counter rollback.

Endpoints:
- POST /id/register|login/(begin|finish)
- POST /id/stepup/(begin|finish)
- POST /id/session/token

## UBL Nuclear
- POST /link/validate (sem efeitos)
- POST /link/commit   (validate + append + NOTIFY)
- GET  /ledger/{container_id}/tail (SSE)

`signing_bytes = version||container_id||expected_sequence||previous_hash||atom_hash||intent_class||physics_delta` (sem pact/author/signature)  
Erros: InvalidVersion, InvalidSignature, InvalidTarget, RealityDrift, SequenceMismatch, PhysicsViolation, PactViolation, UnauthorizedEvolution.

## Storage (MinIO)
Buckets: vault-repos, vault-workspaces, vault-deploy. Disco local (LAB 8GB) = rascunho.

## Runner (LAB 512 ou iPhone)
gVisor/nsjail, egress whitelist, tmpfs quota/TTL, sempre Receipt (Observation).

## “Done if…” (resumo)
- Repo Δ≠0 → PhysicsViolation; Workspace `ws/receipt` no tail; Prod exige two-man rule; Ledger append-only; Auth/step-up OK; JWT Ed25519; Métricas/Logs expostos.
