# LAB 512 → LAB 8GB — Integration Status

**Data:** 26 dez 2025  
**Objetivo:** Sistema Git distribuído com UBL (3 computadores: LOCAL, SERVER, RUNNER)

---

## ✅ INTEGRADO (LAB 512 - Dev Local)

### 1. Código Fonte
```
kernel/rust/ubl-server/src/
├── id_ledger.rs              (placeholder - logs eventos)
├── id_session_token.rs       (JWT Bearer - precisa PEM key)
├── repo_routes.rs            (presign + commit-ref)
├── middleware_require_stepup.rs (placeholder)
└── main.rs                   (routers merged)
```

### 2. SQL Schema
```sql
-- Tabela id_api_token criada
CREATE TABLE id_api_token (
  jti TEXT PRIMARY KEY,
  sid TEXT REFERENCES id_subject(sid),
  aud TEXT NOT NULL,
  scope JSONB NOT NULL,
  flavor TEXT NOT NULL,
  issued_at TIMESTAMPTZ DEFAULT now(),
  expires_at TIMESTAMPTZ NOT NULL
);
```

### 3. Dependencies (Cargo.toml)
```toml
jsonwebtoken = { version = "9", features = ["use_pem"] }
ed25519-dalek = "2"
base64ct = { version = "1", features = ["alloc"] }
once_cell = "1.19"
```

### 4. Endpoints Novos
- ✅ `POST /id/session/token` - Issue JWT Bearer (precisa JWT_ED25519_PEM env)
- ✅ `POST /repo/presign` - MinIO presign URLs via 'mc' tool
- ✅ `POST /repo/commit-ref` - Git ref atoms (placeholder sem ledger)
- ✅ `GET /metrics` - Prometheus (vazio por enquanto)

### 5. Clients TypeScript
```
clients/
├── ts/sdk/     (repo.ts - presign, upload, commitRef)
└── cli/        (ubl repo push/force)
```

### 6. Build Status
```
✅ Compilação: OK (53s)
✅ Servidor: Rodando localhost:8080
✅ Health: {"status":"healthy","version":"2.0.0+postgres"}
```

---

## 🚧 PLACEHOLDERS (Código preparado, lógica faltando)

### id_ledger.rs
```rust
// ATUAL: Apenas loga evento
tracing::info!(event = "person_registered", payload = %json);
return Ok("0x<placeholder_hash>");

// TODO LAB 8GB: Append real ao ledger
let entry = state.ledger.append(LinkDraft {
    version: 1,
    container_id: "C.Identity",
    expected_sequence, previous_hash,
    atom_hash, intent_class: "Observation",
    physics_delta: "0",
    author_pubkey, signature,
}).await?;
```

### repo_routes.rs (commit-ref)
```rust
// ATUAL: Apenas loga e retorna hash fake
tracing::info!(container_id = "repo://acme/billing", ref = "refs/heads/main");
return Ok(Json({ link_hash: "0x<placeholder>" }));

// TODO LAB 8GB: Append real com signatures
```

### id_session_token.rs
```rust
// ATUAL: SID hardcoded
let sid = "ubl:sid:placeholder";

// TODO LAB 8GB: Extrair de cookie ou Authorization header
let token = extract_bearer_or_cookie(&req)?;
let session = session_db::get_valid(&state.pool, &token).await?;
let sid = session.sid;
```

### middleware_require_stepup.rs
```rust
// ATUAL: Pass-through
Ok(next.run(req).await)

// TODO LAB 8GB: Validar flavor=stepup
let session = extract_and_validate_session(&req, &state).await?;
if session.flavor != "stepup" {
    return Err((403, "step-up required"));
}
```

---

## 🎯 ROADMAP LAB 8GB (Distribuído)

### Fase 1: Completar Ledger Atoms (2-3h)
**Objetivo:** Eventos identity no ledger real

1. **Estrutura LinkDraft completa**
   ```rust
   // db.rs já tem a estrutura correta:
   pub struct LinkDraft {
       version: u8,
       container_id: String,
       expected_sequence: i64,
       previous_hash: String,
       atom_hash: String,
       intent_class: String,
       physics_delta: String,  // i128
       author_pubkey: String,  // hex
       signature: String,      // hex
   }
   ```

2. **Implementar id_ledger.rs real**
   - Buscar sequence do container `C.Identity`
   - Construir atom JSON canonical
   - Hash atom → atom_hash
   - Assinar signing_bytes com Ed25519
   - Chamar `state.ledger.append()`

3. **Eventos a emitir**
   - `person_registered` em register_finish
   - `person_authenticated` em login_finish
   - `stepup_granted` em stepup_finish
   - `counter_rollback` em replay attack

4. **Instrumentar métricas**
   - `ID_DECISIONS.with_label_values(&["register","accept",""]).inc()`
   - `WEBAUTHN_OPS.with_label_values(&["login","finish"]).inc()`

### Fase 2: JWT Bearer Auth (1-2h)
**Objetivo:** CLI/Runner auth sem cookies

1. **Gerar Ed25519 key**
   ```bash
   openssl genpkey -algorithm ED25519 -out jwt-key.pem
   export JWT_ED25519_PEM="$(cat jwt-key.pem)"
   export JWT_KID="ubl-ed25519-prod-v1"
   ```

2. **Extrair SID real em id_session_token**
   ```rust
   // De cookie (browsers)
   let cookie = req.headers().get(COOKIE)?;
   let token = parse_cookie(cookie, "session")?;
   
   // Ou de Bearer (CLI/Runner)
   let auth = req.headers().get(AUTHORIZATION)?;
   let token = auth.strip_prefix("Bearer ")?;
   
   let session = session_db::get_valid(&state.pool, token).await?;
   ```

3. **Validar scope admin → step-up**
   ```rust
   if body.scope.contains(&"admin".to_string()) && session.flavor != "stepup" {
       return Err((403, "step-up required for admin scope"));
   }
   ```

4. **Gravar JTI para auditoria**
   ```sql
   INSERT INTO id_api_token (jti, sid, aud, scope, flavor, expires_at)
   VALUES ($1, $2, $3, $4, $5, to_timestamp($6));
   ```

### Fase 3: Repo Git Distribuído (2-3h)
**Objetivo:** Push/pull repos entre LOCAL ↔ SERVER

1. **MinIO setup**
   ```bash
   # LAB 8GB (SERVER)
   docker run -d -p 9000:9000 -p 9001:9001 \
     -e MINIO_ROOT_USER=admin \
     -e MINIO_ROOT_PASSWORD=<strong> \
     minio/minio server /data --console-address ":9001"
   
   # Configure mc alias
   mc alias set ubl https://s3.ubl.internal admin <password>
   mc mb ubl/vault-repos
   ```

2. **Env vars (SERVER)**
   ```bash
   export MINIO_ALIAS=ubl
   export MINIO_BUCKET_REPOS=vault-repos
   export MINIO_ENDPOINT=https://s3.ubl.internal
   ```

3. **Implementar repo_routes commit-ref real**
   - Construir atom `git/ref` canonical (BTreeMap sorted keys)
   - Hash → atom_hash
   - Buscar sequence de `repo://tenant/repo`
   - Assinar signing_bytes
   - Append ao ledger

4. **CLI push workflow**
   ```bash
   # LOCAL
   export UBL_SERVER_URL=https://gateway.ubl.internal
   
   ubl repo push \
     --tenant acme \
     --repo billing \
     --ref refs/heads/main \
     --dir ./src \
     --sid $(ubl auth token)
   ```

### Fase 4: Runner Remoto (3-4h)
**Objetivo:** Sandbox execution em computador separado

1. **Setup RUNNER machine**
   ```yaml
   # templates/runner.lab512.yaml
   runner:
     listen: "0.0.0.0:9090"
     sandbox: "gvisor"
     egress_allow: ["registry.acme.local:443"]
     tmpfs_quota_mb: 512
     receipts_dir: "/opt/ubl/receipts"
   ```

2. **Runner auth (machine identity)**
   ```bash
   # RUNNER obtém JWT próprio via client_credentials
   export RUNNER_CLIENT_ID=runner-01
   export RUNNER_CLIENT_SECRET=<secret>
   ```

3. **Tail ledger via SSE**
   ```rust
   // RUNNER poll ledger
   let mut stream = client
       .get("https://gateway.ubl.internal/ledger/repo://acme/billing/tail")
       .bearer_auth(&jwt)
       .send()
       .await?
       .bytes_stream();
   
   while let Some(event) = stream.next().await {
       if event.data.contains("git/ref") {
           execute_deploy(event).await?;
       }
   }
   ```

4. **Pull objects do MinIO**
   ```rust
   // RUNNER pull SHA256 objects
   for obj in manifest.objects {
       let path = format!("vault-repos/{tenant}/{repo}/objects/{prefix}/{sha}");
       let data = minio.get_object(&path).await?;
       fs::write(format!("./workspace/{sha}"), data)?;
   }
   ```

5. **Execute em sandbox**
   ```bash
   # gvisor
   runsc --root /tmp/runsc run \
     --network none \
     --tmpfs /tmp:size=512m \
     billing-deploy
   ```

---

## 📋 ENV VARS (3 Computadores)

### LOCAL (Dev Mac)
```bash
export UBL_SERVER_URL=https://gateway.ubl.internal
export DATABASE_URL=postgres://localhost/ubl_dev  # apenas para testes locais
```

### SERVER (Gateway Remoto)
```bash
export DATABASE_URL=postgres://ubl_prod@10.8.0.10:5432/ubl_prod
export MINIO_ENDPOINT=https://s3.ubl.internal
export MINIO_ALIAS=ubl
export MINIO_BUCKET_REPOS=vault-repos
export JWT_ED25519_PEM="$(cat /etc/ubl/jwt-key.pem)"
export JWT_KID=ubl-ed25519-prod-v1
export WEBAUTHN_RP_ID=gateway.ubl.internal
export WEBAUTHN_ORIGIN=https://gateway.ubl.internal
```

### RUNNER (Sandbox Remoto)
```bash
export UBL_GATEWAY_URL=https://gateway.ubl.internal
export UBL_RUNNER_ID=runner-01
export UBL_SANDBOX=gvisor
export RUNNER_CLIENT_SECRET=<secret>
```

---

## 🔍 Testes Integração

### Test 1: JWT Auth Flow
```bash
# 1. Login WebAuthn
curl -X POST https://gateway.ubl.internal/id/login/begin \
  -d '{"username":"alice"}'

# 2. Finish (com authenticator)
curl -X POST https://gateway.ubl.internal/id/login/finish \
  -d '{"challenge_id":"...","credential":{...}}'
# → Set-Cookie: session=<token>; HttpOnly; Secure

# 3. Issue JWT Bearer
curl -X POST https://gateway.ubl.internal/id/session/token \
  -H "Cookie: session=<token>" \
  -d '{"aud":"ubl://cli","scope":["read","write"]}'
# → {"access_token":"eyJ...","expires_in":3600}
```

### Test 2: Repo Push
```bash
# Setup
export JWT=<access_token>
mkdir test-repo && echo "hello" > test-repo/index.html

# Push
ubl repo push \
  --tenant acme \
  --repo test \
  --ref refs/heads/main \
  --dir ./test-repo \
  --base-url https://gateway.ubl.internal \
  --sid "$JWT"

# Verify ledger
curl https://gateway.ubl.internal/ledger/repo://acme/test/tail \
  -H "Authorization: Bearer $JWT"
# → SSE stream com atom git/ref
```

### Test 3: Runner Pull & Execute
```bash
# RUNNER subscribes
./runner \
  --gateway https://gateway.ubl.internal \
  --client-id runner-01 \
  --watch repo://acme/test

# Detecta novo commit → pull objects → execute
# Output: Receipt em /opt/ubl/receipts/<link_hash>.json
```

---

## 📊 Métricas Esperadas

```prometheus
# Identity
ubl_id_decision_total{operation="login",decision="accept",error_code=""} 42
ubl_webauthn_operations_total{operation="register",phase="begin"} 15
ubl_rate_limit_rejections_total{operation="login"} 3
ubl_progressive_lockout_total{failure_count="6"} 1

# Repo
ubl_repo_commits_total{tenant="acme",repo="billing"} 128
ubl_repo_presign_requests_total{tenant="acme"} 512

# Runner
ubl_runner_executions_total{runner_id="runner-01",status="success"} 100
ubl_runner_sandbox_time_seconds{runner_id="runner-01"} 1234.56
```

---

## 🎓 Conceitos UBL

### Container IDs
- `C.Identity` → eventos identity (person_registered, etc)
- `repo://{tenant}/{repo}` → git refs e objects
- `workspace://{tenant}/{ws}` → workspaces dev
- `deploy://{tenant}/{app}` → deployments prod

### Intent Classes (Δ)
- **Observation (Δ=0)**: Imutável, sem custo energético (git refs, identity logs)
- **Conservation (Δ=1)**: State change conservativo (update config)
- **Entropy (Δ=-1)**: Destruição controlada (delete resource)
- **Evolution (Δ=+N)**: Criação complexa (new service)

### Ledger Tangency
```
previous_hash → entry_hash
     ↓              ↓
 signing_bytes = canonical(version, container, seq, prev, atom, intent, delta, pubkey)
     ↓
 signature = Ed25519.sign(signing_bytes)
```

### TDLN Policies (WASM)
- `tdln.repo.wasm` - Quem pode push? Force push precisa 2 approvals
- `tdln.workspace.wasm` - Isolamento tenant, quotas
- `tdln.deploy.wasm` - Canary releases, rollback automático

---

## 📝 Notas Implementação

1. **Hardcoded localhost removidos**: Preparado para env vars
2. **Placeholders documentados**: TODOs claros para LAB 8GB
3. **Estrutura compatível**: `db::LinkDraft` já correto
4. **Build limpo**: 53s, sem errors
5. **Clients prontos**: SDK/CLI apenas esperando server endpoints completos

**Próximo commit:** Quando estiver em LAB 8GB, começar por Fase 1 (Ledger Atoms).
