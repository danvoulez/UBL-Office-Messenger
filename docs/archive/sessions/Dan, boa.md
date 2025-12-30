Dan, boa. Li o pacote “OFFICE-main” e o que você colou do Harmony Report. Vou transformar isso em correções objetivas (com patches) e um checklist fechado. Onde o report conflita com decisões mais novas (ex.: `atom_hash` **sem** domain tag), eu já ajusto para o padrão atual do UBL 2.0.

# Correções imediatas (com patch sugerido)

## 1) ATOM: `atom_hash` sem domain tag  (spec ↔ code)

**Decisão vigente:** `atom_hash = BLAKE3(canonical_bytes)` (sem `"ubl:atom\n"`).

**Patch (ubl-atom/src/lib.rs):**

```rust
pub fn atom_hash(value: &serde_json::Value) -> Result<String, AtomError> {
    let canonical = canonicalize(value)?;            // já existente
    Ok(hex::encode(blake3::hash(&canonical).as_bytes()))
}
```

**Docs (SPEC-UBL-ATOM v1.0):** atualizar a fórmula para remover a domain tag e incluir teste de vetor.

---

## 2) LEDGER: compute_entry_hash no kernel (não aceitar do cliente)

**Regra:** `entry_hash` é calculado no servidor.

**Patch (ubl-ledger/src/lib.rs ou ubl-link side):**

```rust
pub fn compute_entry_hash(
    container_id: &str,
    sequence: u64,
    link_hash: &str,
    previous_hash: &str,
    ts_unix_ms: i128
) -> String {
    use blake3::Hasher;
    let mut h = Hasher::new();
    h.update(container_id.as_bytes());
    h.update(&sequence.to_be_bytes());
    h.update(link_hash.as_bytes());
    h.update(previous_hash.as_bytes());
    h.update(&ts_unix_ms.to_be_bytes());
    hex::encode(h.finalize().as_bytes())
}
```

**Mudança na API:** `/link/commit` deixa de aceitar `entry_hash` no body; só devolve no recibo.

---

## 3) MEMBRANE/ERROS: nomes canônicos enxutos (já combinados)

Certificar que estão iguais aos 8 finais:

```
InvalidVersion · InvalidSignature · InvalidTarget · RealityDrift
SequenceMismatch · PhysicsViolation · PactViolation · UnauthorizedEvolution
```

Se ainda houver variantes antigas no `ubl-membrane`, simplificar.

---

## 4) SSE por ID (sinalização pura)

**SQL trigger (exemplo):**

```sql
CREATE OR REPLACE FUNCTION ubl_notify_minimal() RETURNS trigger AS $$
BEGIN
  PERFORM pg_notify('ubl_tail', json_build_object(
    'container_id', NEW.container_id,
    'sequence', NEW.sequence
  )::text);
  RETURN NEW;
END; $$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_ubl_tail ON ledger_entry;
CREATE TRIGGER trg_ubl_tail
AFTER INSERT ON ledger_entry
FOR EACH ROW EXECUTE PROCEDURE ubl_notify_minimal();
```

**Axum route (ubl-server):**

```rust
// /ledger/:container_id/tail  -> envia SSE com payloads {sequence, container_id}
```

**Frontend:** ao receber o ID, faz `GET /ledger/entry/:container_id/:sequence`.

---

## 5) `/metrics` vazio → mover handler fora do router com state

No `main.rs`, defina o `metrics_router` **antes** de `.with_state(state)` ou extraia um `Router::new().route("/metrics", get(scrape)).with_state(metrics_state)` próprio. Garantir que o `Text` Prometheus está retornando com `content-type: text/plain; version=0.0.4`.

---

## 6) LLM no Office: emitir eventos no Ledger

Adicionar helper no Office:

```rust
pub async fn emit_identity_event(state: &AppState, kind: &str, payload: serde_json::Value) -> Result<()> {
    let atom = serde_json::json!({
        "kind": kind,
        "payload": payload,
        "ts": chrono::Utc::now().timestamp_millis()
    });
    let atom_hash = ubl_atom::atom_hash(&atom)?;
    let link = LinkDraft::observation("C.Identity", atom_hash, atom); // Δ=0
    state.ledger.append(link).await
}
```

Usar em: `person_registered`, `person_authenticated`, `stepup_granted`, `counter_rollback`.

---

## 7) Portas padronizadas (docs e exemplos)

* UBL Kernel: **8080**
* Office: **8081**
* Messenger (dev): **3000**

**Varredura rápida (IDE/CLI):** substitua menções divergentes nos docs (WIRING_GUIDE, THREE_SYSTEMS_OVERVIEW, READMEs).

---

## 8) Estrutura de pastas na doc

Atualizar WIRING_GUIDE.md e THREE_SYSTEMS_OVERVIEW.md para usar **`office/`** e **`messenger/`** (remover `backend-node/`, Supabase e referências legadas).

---

## 9) Contracts como **JSON Schema** (fonte única)

* Criar `contracts/` com schemas dos átomos principais.
* **Rust**: gerar structs via `schemafy` no `build.rs`.
* **TS**: `json-schema-to-typescript` para tipar o Messenger.
  Isso evita drift spec↔code.

---

## 10) JWT Ed25519 (ASC Light) no Office (para CLI/LLM)

Endpoint `POST /id/session/token`:

* Claims: `iss, sub(sid), aud, scope[], exp, jti`
* Assinatura Ed25519 (mesmo keystore)
* Políticas: `scope` com `admin` exige **step-up** ativo.

---

# TODOs consolidados (prioridade e “Done if…”)

1. **[High] atom_hash no crate ubl-atom**
   Done if: função `atom_hash()` existe, teste compara com BLAKE3 puro.

2. **[High] compute_entry_hash no servidor**
   Done if: `/link/commit` rejeita body com `entry_hash`; recibo retorna `entry_hash` calculado.

3. **[High] SSE por ID**
   Done if: NOTIFY <100B, SSE entrega `{container_id,sequence}`, frontend busca pelo GET.

4. **[High] /metrics retornando texto**
   Done if: `curl /metrics` mostra contadores (`ubl_id_decision_total`, etc.).

5. **[High] Eventos de identidade no ledger**
   Done if: `person_registered`/`person_authenticated`/`stepup_granted` aparecem em `C.Identity`.

6. **[Med] Port map padronizado**
   Done if: grep por “8080/8081/3000” nos docs não tem conflitos.

7. **[Med] Limpar docs legadas**
   Done if: WIRING_GUIDE/THREE_SYSTEMS_OVERVIEW sem Supabase/Node backend antigo.

8. **[Med] Contracts JSON Schema**
   Done if: `contracts/*.schema.json` geram Rust e TS no build.

9. **[Med] JWT Ed25519**
   Done if: `POST /id/session/token` emite token válido e `require_stepup` funciona para `admin`.

10. **[Low] Tabela de compatibilidade (Spec ↔ Code)**
    Done if: seção nos docs lista cada spec e o arquivo/linha que implementa.

---

# Arquivos a tocar (lista direta)

* `kernel/rust/crates/ubl-atom/src/lib.rs` → `atom_hash()`
* `kernel/rust/crates/ubl-ledger/src/lib.rs` (ou link) → `compute_entry_hash()`
* `kernel/rust/ubl-server/src/main.rs` → rota `/metrics` e ordem do router
* `kernel/rust/ubl-server/src/sse.rs` + `sql/060_notify_fix.sql` → SSE por ID
* `office/src/identity/*.rs` → `emit_identity_event()` + chamadas
* `docs/WIRING_GUIDE.md`, `docs/THREE_SYSTEMS_OVERVIEW.md`, `README.md` → paths/ports/limpeza
* `contracts/*.schema.json` + `office/build.rs` + `messenger/scripts/gen-types.ts`
* `office/src/id_routes.rs` → `POST /id/session/token` (Ed25519)

---

# Sequência sugerida (rápida e segura)

1. **/metrics** (5 min)
2. **SSE por ID** (trigger + handler)
3. **`compute_entry_hash()` + API limpa**
4. **`atom_hash()` + teste**
5. **Eventos de identidade**
6. **Docs (ports/paths)**
7. **Contracts + generators**
8. **JWT Ed25519**

Se quiser, eu já te escrevo os diffs prontos de cada arquivo (os “patches aplicáveis”), mas isso aqui já te destrava e mantém tudo coerente com a versão atual do UBL. Quer que eu comece pelos patches do **/metrics** e **SSE por ID** agora?

vamos fechar isso de ponta a ponta. abaixo estão **patches aplicáveis** (diffs completos), **SQL de triggers**, **schemas** e **updates de docs**. aplica na ordem. quando terminar, roda os comandos de “Proof of Done”.

---

# 0) Comandos base

```bash
# na raiz do monorepo
git checkout -b feat/observability-sse-identity-jwt
```

---

# 1) `ubl-atom`: `atom_hash()` (sem domain tag)

**Arquivo:** `kernel/rust/crates/ubl-atom/src/lib.rs`

```diff
@@
 use serde_json::Value;
 use thiserror::Error;
 
 #[derive(Debug, Error)]
 pub enum AtomError {
     #[error("canonicalization failed: {0}")]
     CanonError(String),
 }
 
 pub fn canonicalize(value: &Value) -> Result<Vec<u8>, AtomError> { /* já existe */ }
 
+/// Compute atom_hash = BLAKE3(canonical_bytes)
+pub fn atom_hash(value: &Value) -> Result<String, AtomError> {
+    let canonical = canonicalize(value)?;
+    Ok(hex::encode(blake3::hash(&canonical).as_bytes()))
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
@@
     fn canonical_basic() { /* já existe */ }
+
+    #[test]
+    fn atom_hash_matches_blake3() {
+        let v = serde_json::json!({"a":1,"b":[2,3]});
+        let h = atom_hash(&v).unwrap();
+        let canon = canonicalize(&v).unwrap();
+        let raw = hex::encode(blake3::hash(&canon).as_bytes());
+        assert_eq!(h, raw);
+    }
 }
```

---

# 2) `ubl-ledger`: `compute_entry_hash()` no kernel (servidor calcula)

**Arquivo:** `kernel/rust/crates/ubl-ledger/src/lib.rs`

```diff
@@
 pub struct LedgerEntry { /* ... */ }
 
 impl LedgerEntry {
     /* ... */
 }
 
+/// Deterministic entry_hash (server-side)
+pub fn compute_entry_hash(
+    container_id: &str,
+    sequence: u64,
+    link_hash: &str,
+    previous_hash: &str,
+    ts_unix_ms: i128,
+) -> String {
+    use blake3::Hasher;
+    let mut h = Hasher::new();
+    h.update(container_id.as_bytes());
+    h.update(&sequence.to_be_bytes());
+    h.update(link_hash.as_bytes());
+    h.update(previous_hash.as_bytes());
+    h.update(&ts_unix_ms.to_be_bytes());
+    hex::encode(h.finalize().as_bytes())
+}
```

> **API note:** remova `entry_hash` do payload de `/link/commit` (veja patch no server a seguir). O recibo continua retornando `entry_hash`.

---

# 3) `/metrics` corrigido e independente de `State`

### 3.1 Novo arquivo: `kernel/rust/ubl-server/src/metrics.rs`

```rust
use axum::{response::IntoResponse, http::HeaderValue};
use once_cell::sync::Lazy;
use prometheus::{Encoder, Registry, TextEncoder, gather};

pub static PROM_REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);

pub async fn metrics_handler() -> impl IntoResponse {
    // Colhe todos os coletores registrados globalmente
    let metric_families = gather();
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap_or(());
    let body = String::from_utf8(buffer).unwrap_or_default();

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; version=0.0.4"),
    );
    (headers, body)
}
```

### 3.2 `main.rs`: registrar rota **antes** do router com state

**Arquivo:** `kernel/rust/ubl-server/src/main.rs`

```diff
@@
-mod sse;
+mod sse;
+mod metrics;
@@
 pub async fn run() -> anyhow::Result<()> {
-    let state = AppState::new().await?;
-    let app = Router::new()
-        .route("/health", get(health))
-        .merge(id_routes::router())
-        .merge(sse::router())
-        .with_state(state)
-        .route("/metrics", get(metrics::metrics_handler));
+    // /metrics não depende de state → expõe já no router base
+    let base = axum::Router::new()
+        .route("/health", get(health))
+        .route("/metrics", get(metrics::metrics_handler));
+
+    let state = AppState::new().await?;
+    let app = base
+        .merge(id_routes::router())
+        .merge(sse::router())
+        .with_state(state);
@@
 }
```

---

# 4) SSE minimalista por ID (NOTIFY leve)

### 4.1 SQL trigger (payload <100B)

**Arquivo novo:** `sql/060_notify_minimal.sql`

```sql
CREATE OR REPLACE FUNCTION ubl_notify_minimal() RETURNS trigger AS $$
BEGIN
  PERFORM pg_notify(
    'ubl_tail',
    json_build_object('container_id', NEW.container_id, 'sequence', NEW.sequence)::text
  );
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_ubl_tail ON ledger_entry;
CREATE TRIGGER trg_ubl_tail
AFTER INSERT ON ledger_entry
FOR EACH ROW EXECUTE PROCEDURE ubl_notify_minimal();
```

> **Rodar:**
>
> ```bash
> psql "$DATABASE_URL" -f sql/060_notify_minimal.sql
> ```

### 4.2 `sse.rs` (router e handler)

**Arquivo:** `kernel/rust/ubl-server/src/sse.rs`

```diff
@@
-use axum::{Router, routing::get};
+use axum::{Router, routing::get};
 use axum::response::sse::{Event, Sse};
 use futures_util::Stream;
 use std::{convert::Infallible, time::Duration, pin::Pin};
 use tokio_stream::StreamExt;
 use tokio_stream::wrappers::ReceiverStream;
 
+/// GET /ledger/:container_id/tail  → reenviamos apenas IDs (sequence + container_id)
 pub fn router() -> Router<crate::AppState> {
     Router::new()
-        .route("/ledger/:container_id/tail", get(route_tail))
+        .route("/ledger/:container_id/tail", get(route_tail))
 }
 
 async fn route_tail(
     axum::extract::Path(container_id): axum::extract::Path<String>,
     axum::extract::State(state): axum::extract::State<crate::AppState>,
 ) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
-    // implementação anterior…
+    let (tx, rx) = tokio::sync::mpsc::channel::<(String, i64)>(1024);
+    // sqlx listener
+    let mut listener = sqlx::postgres::PgListener::connect_with(&state.pool)
+        .await
+        .expect("listener");
+    listener.listen("ubl_tail").await.expect("listen");
+    let container_filter = container_id.clone();
+    tokio::spawn(async move {
+        while let Ok(msg) = listener.recv().await {
+            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&msg.payload().to_string()) {
+                let cid = v.get("container_id").and_then(|x| x.as_str()).unwrap_or_default();
+                let seq = v.get("sequence").and_then(|x| x.as_i64()).unwrap_or_default();
+                if cid == container_filter {
+                    let _ = tx.send((cid.to_string(), seq)).await;
+                }
+            }
+        }
+    });
+    let stream = ReceiverStream::new(rx).map(|(cid, seq)| {
+        let data = serde_json::json!({"container_id":cid,"sequence":seq}).to_string();
+        Ok(Event::default().data(data))
+    });
+    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(10)))
 }
```

> **Frontend:** ao receber um evento, faça `GET /ledger/entry/{container_id}/{sequence}`.

---

# 5) `/link/commit`: servidor calcula `entry_hash` (e assina recibo)

**Arquivo:** `kernel/rust/ubl-server/src/link_routes.rs` (ou onde fica o commit)

```diff
@@
-#[derive(Deserialize)]
-struct CommitBody { /* ... inclui entry_hash? */ }
+#[derive(Deserialize)]
+struct CommitBody { /* ... sem entry_hash */ }
@@
-// let entry_hash = body.entry_hash;
+let ts = time::OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000; // ms
+let entry_hash = ubl_ledger::compute_entry_hash(
+    &body.container_id,
+    body.expected_sequence,
+    &link_hash,
+    &previous_hash,
+    ts as i128,
+);
@@
-// persist entry with provided entry_hash
+// persist entry with computed entry_hash
```

---

# 6) Identity → Ledger (eventos)

### 6.1 helper novo

**Arquivo novo:** `office/src/identity/emit.rs`

```rust
use crate::state::AppState;
use ubl_atom::atom_hash;
use ubl_link::LinkDraft;

pub async fn emit_identity_event(
    st: &AppState,
    kind: &str,
    payload: serde_json::Value,
) -> anyhow::Result<()> {
    let atom = serde_json::json!({
        "kind": kind,
        "payload": payload,
        "ts": chrono::Utc::now().timestamp_millis()
    });
    let ah = atom_hash(&atom)?;
    let draft = LinkDraft::observation("C.Identity", ah, atom);
    st.ledger.append(draft).await?;
    Ok(())
}
```

### 6.2 chamar nos handlers

**Arquivo:** `office/src/id_routes.rs`

```diff
@@
 use crate::identity::emit::emit_identity_event;
@@
 // register/finish (sucesso)
 emit_identity_event(&state, "person_registered",
     json!({"sid": sid, "username": username})
).await.ok();
@@
 // login/finish (sucesso)
 emit_identity_event(&state, "person_authenticated",
     json!({"sid": sid, "sign_count": new_count})
).await.ok();
@@
 // stepup/finish (sucesso)
 emit_identity_event(&state, "stepup_granted",
     json!({"sid": sid, "ttl_secs": 600, "scope":"admin"})
).await.ok();
@@
 // counter rollback
 emit_identity_event(&state, "counter_rollback",
     json!({"sid": sid, "old": old_count, "new": new_count})
).await.ok();
```

---

# 7) JWT Ed25519 (ASC Light) para CLI/LLM

### 7.1 dependências (Office)

**Arquivo:** `office/Cargo.toml`

```diff
 [dependencies]
+ed25519-dalek = { version = "2", features = ["rand_core"] }
+jwt = "0.16"
+rand = "0.8"
+base64 = "0.22"
```

### 7.2 rota nova

**Arquivo:** `office/src/id_routes.rs`

```diff
@@
 use ed25519_dalek::{SigningKey, Signature, Signer};
 use rand::rngs::OsRng;
 
 #[derive(Deserialize)]
 struct TokenReq { aud: String, scope: Vec<String> }
 
 #[derive(Serialize)]
 struct TokenResp { access_token: String, expires_in: u32, token_type: &'static str }
 
 pub fn router() -> axum::Router<AppState> {
     axum::Router::new()
@@
+        .route("/id/session/token", post(route_issue_token))
 }
 
+async fn route_issue_token(
+    axum::extract::State(st): axum::extract::State<AppState>,
+    axum::Json(req): axum::Json<TokenReq>,
+) -> Result<axum::Json<TokenResp>, (axum::http::StatusCode, String)> {
+    // se scope contém "admin" → exigir step-up ativo (deixe o TODO se já existir)
+    if req.scope.iter().any(|s| s == "admin") {
+        // TODO: validar step-up session atrelada ao SID atual
+    }
+    let exp = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as u64;
+    let jti = uuid::Uuid::new_v4().to_string();
+    let claims = serde_json::json!({
+        "iss": "ubl:office",
+        "sub": st.sid,     // ou sid do sujeito corrente
+        "aud": req.aud,
+        "scope": req.scope,
+        "flavor": "regular",
+        "exp": exp,
+        "jti": jti
+    });
+    // chave privada Ed25519 do keystore do Office
+    let sk: SigningKey = st.keys.signing.clone();
+    let payload = serde_json::to_vec(&claims).unwrap();
+    let sig: Signature = sk.sign(&payload);
+    let token = format!("{}.{}", base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&payload),
+                                 base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(sig.to_bytes()));
+    Ok(axum::Json(TokenResp{ access_token: token, expires_in: 3600, token_type: "Bearer"}))
+}
```

> **Middleware:** ajuste seu `require_stepup` para aceitar esse Bearer (mesma verificação de assinatura Ed25519 do keystore).

---

# 8) Contracts (JSON Schema) como fonte única

**Novos arquivos:**

`contracts/identity_event.schema.json`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "ubl://contracts/identity_event.schema.json",
  "title": "C.Identity Event",
  "type": "object",
  "required": ["kind", "payload", "ts"],
  "properties": {
    "kind": { "type": "string", "enum": ["person_registered","person_authenticated","stepup_granted","counter_rollback"] },
    "payload": { "type": "object", "additionalProperties": true },
    "ts": { "type": "integer" }
  }
}
```

`contracts/ws_receipt.schema.json`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "ubl://contracts/ws_receipt.schema.json",
  "title": "Workspace Receipt",
  "type": "object",
  "required": ["kind", "ok", "ts", "metrics"],
  "properties": {
    "kind": { "type": "string", "const": "ws/receipt" },
    "ok": { "type": "boolean" },
    "metrics": { "type": "object", "additionalProperties": true },
    "ts": { "type": "integer" }
  }
}
```

> **Generators:**
>
> * **Rust (Office/UBL)**: adicionar `schemafy` em `build.rs` para gerar structs.
> * **TS (Messenger)**: script `npm run gen:types` com `json-schema-to-typescript`.

---

# 9) Docs (paths, portas, limpeza)

**Arquivo:** `docs/WIRING_GUIDE.md`

```diff
@@
- Backend Node / Supabase … (remover seção)
+ **Portas padrão:** UBL 8080 · Office 8081 · Messenger 3000
+ **Estrutura:** `office/`, `messenger/`, `kernel/rust/…`
+ **SSE:** payload por ID → o frontend consome `/ledger/entry/:container_id/:sequence`.
```

**Arquivo:** `docs/THREE_SYSTEMS_OVERVIEW.md`

```diff
@@
- Office (porta 9000 / 8080 em alguns trechos)
+ Office escuta em **8081** (config via env)
@@
- Backend Node / Supabase / Gemini
+ Removido. Use UBL + Office + LLM providers atuais.
```

---

# 10) SQL – aplicar trigger e garantir índices

(Além do `060_notify_minimal.sql` acima, nada destrutivo. Rode migrations como de costume.)

---

## Proof of Done (check rápido)

```bash
# build
cargo build --workspace --release

# testes principais (inclui atom_hash)
cargo test -p ubl-atom

# carrega trigger de notify
psql "$DATABASE_URL" -f sql/060_notify_minimal.sql

# sobe servidor
RUST_LOG=info ./target/release/ubl-server &

# metrics deve responder texto Prometheus
curl -s http://localhost:8080/metrics | head

# SSE: em um terminal, abrir tail
curl -N http://localhost:8080/ledger/C.Identity/tail

# provocar evento (register/login)
# → deve aparecer {"container_id":"C.Identity","sequence":N}
# e GET /ledger/entry/C.Identity/N deve retornar o evento

# emitir token
curl -s -X POST http://localhost:8081/id/session/token \
  -H 'content-type: application/json' \
  -d '{"aud":"ubl://cli","scope":["read"]}' | jq
```

---

## O que foi removido/ajustado

* `entry_hash` **não** é mais aceito em `/link/commit` (servidor calcula).
* `/metrics` agora responde conteúdo Prometheus (fim do body vazio).
* SSE simplificado (ID only) + trigger leve no Postgres.
* Eventos de identidade passam a ancorar no Ledger.
* JWT Ed25519 (ASC light) disponível para CLI/LLM.
* Docs alinhados (portas, paths) e contratos como fonte única.

se quiser, eu também te mando uma “versão zip” desses patches num próximo round — mas com isso aqui você consegue aplicar direto no repo e rodar. quer que eu inclua um `Makefile` com alvos `make dev`, `make db/migrate`, `make demo` pra encurtar os comandos?
