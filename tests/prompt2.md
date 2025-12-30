perfeito — aqui vai o “pacotinho” que você pediu agora: types TS já gerados a partir dos três schemas (ws.test.request, ws.receipt, deploy.request) + teste de integração do Office que valida 202 + receipt (com UBL “mockado”). Na próxima mensagem eu mando o mesmo conjunto “o que mudar/adicionar” mas com a variação de Unix Socket (UBL/Office/Postgres sem TCP).

⸻

1) Tipos TypeScript (pré-gerados)

Coloque estes arquivos no repo:

clients/types/ubl/atoms/ws.test.request.d.ts
clients/types/ubl/atoms/ws.receipt.d.ts
clients/types/ubl/atoms/deploy.request.d.ts
clients/types/ubl/index.d.ts

clients/types/ubl/atoms/ws.test.request.d.ts

export interface WsLimits {
  cpu?: number;
  mem_mb?: number;
  timeout_sec?: number;
  net?: boolean;
}

export interface WsTestRequest {
  kind: "ws/test/request";
  tenant: string;
  workspace: string;
  repo: string;
  sha: string;
  suite: string;
  limits?: WsLimits;
}

clients/types/ubl/atoms/ws.receipt.d.ts

export interface ReceiptArtifact {
  name: string;
  hash: string;
}

export interface WsReceipt {
  kind: "ws/receipt";
  tenant: string;
  workspace: string;
  trigger: string;       // deve casar com o atom_hash do request
  exit_code: number;
  duration_ms: number;
  stdout_hash?: string;
  stderr_hash?: string;
  artifacts?: ReceiptArtifact[];
  toolchain?: Record<string, unknown>;
}

clients/types/ubl/atoms/deploy.request.d.ts

export type DeployStrategy = "canary" | "blue-green" | "rolling";
export type DeployEnv = "dev" | "stg" | "prod";

export interface DeployRequest {
  kind: "deploy/request";
  tenant: string;
  app: string;
  env: DeployEnv;
  image_digest: string;
  strategy: DeployStrategy;
  desired_replicas?: number;
}

clients/types/ubl/index.d.ts

export * from "./atoms/ws.test.request";
export * from "./atoms/ws.receipt";
export * from "./atoms/deploy.request";

Opcional (para automatizar geração no futuro):
npm i -D json-schema-to-typescript e um script schema:gen apontando contracts/ubl/atoms/*.schema.json → clients/types/ubl/atoms/*.d.ts.
Mas, por agora, os arquivos acima já estão prontos para uso.

⸻

2) Office pronto para teste de integração

Para facilitar o teste, exponha um builder de app no Office.

Novo kernel/rust/ubl-office/src/lib.rs

pub mod types;
pub mod asc;
pub mod routes { pub mod ws; pub mod deploy; }

use axum::{Router, routing::get};

pub fn app(ubl_base: String) -> Router {
    let ws_router = routes::ws::router(routes::ws::OfficeState { ubl_base: ubl_base.clone() });
    let deploy_router = routes::deploy::router(routes::deploy::OfficeState { ubl_base });

    Router::new()
        .merge(ws_router)
        .merge(deploy_router)
        .route("/health", get(|| async { axum::Json(serde_json::json!({"ok":true})) }))
}

Ajustar kernel/rust/ubl-office/src/main.rs

use axum::Router;

#[tokio::main]
async fn main() {
    let ubl_base = std::env::var("UBL_BASE").unwrap_or_else(|_| "http://localhost:8080".into());
    let app: Router = ubl_office::app(ubl_base);
    let addr = std::net::SocketAddr::from(([0,0,0,0], 8081));
    println!("Office listening on http://{addr}");
    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}

Nota: Mantém seu routes/ws.rs, routes/deploy.rs e types.rs iguais aos que definimos.

⸻

3) Teste de Integração (UBL “mockado” + Office real)

Arquivo de teste que sobe um UBL fake (Axum) e o Office real no mesmo runtime.
O UBL fake aceita POST /link/commit e, quando o Office fizer o poll do ledger, responde com um ws/receipt cujo trigger casa com o atom_hash do request.

Crie:

kernel/rust/ubl-office/tests/office_integration.rs

kernel/rust/ubl-office/tests/office_integration.rs

use axum::{Router, routing::{post, get}, Json, extract::Path};
use serde_json::{json, Value};
use std::{net::SocketAddr, sync::{Arc, Mutex}};
use tokio::task::JoinHandle;
use hyper::Server;

#[derive(Default, Clone)]
struct MockState {
    last_atom_hash: Arc<Mutex<Option<String>>>,
}

async fn mock_link_commit(
    axum::extract::State(state): axum::extract::State<MockState>,
    Json(payload): Json<Value>
) -> Json<Value> {
    // payload deve conter atom_hash (LinkCommit assinado pelo Office)
    let atom_hash = payload.get("atom_hash").and_then(|v| v.as_str()).unwrap_or_default().to_string();
    *state.last_atom_hash.lock().unwrap() = Some(atom_hash);
    Json(json!({"ok": true}))
}

async fn mock_ledger_latest(
    Path(container_id): Path<String>,
    axum::extract::State(state): axum::extract::State<MockState>,
) -> Json<Value> {
    let trigger = state.last_atom_hash.lock().unwrap().clone().unwrap_or_else(|| "none".into());
    // sempre devolve um recibo coerente
    let receipt = json!({
        "sequence": 42,
        "atom": {
            "kind": "ws/receipt",
            "tenant": "acme",
            "workspace": "dan-42",
            "trigger": trigger,
            "exit_code": 0,
            "duration_ms": 1234
        }
    });
    Json(Value::Array(vec![receipt]))
}

fn spawn_mock_ubl() -> (String, JoinHandle<()>) {
    let state = MockState::default();
    let app = Router::new()
        .route("/link/commit", post(mock_link_commit))
        .route("/ledger/:container_id/latest", get(mock_ledger_latest))
        .with_state(state);

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let base = format!("http://{}", addr);

    let handle = tokio::spawn(async move {
        Server::from_tcp(listener).unwrap().serve(app.into_make_service()).await.unwrap();
    });

    (base, handle)
}

fn spawn_office(ubl_base: String) -> (String, JoinHandle<()>) {
    let app = ubl_office::app(ubl_base);
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let base = format!("http://{}", addr);

    let handle = tokio::spawn(async move {
        Server::from_tcp(listener).unwrap().serve(app.into_make_service()).await.unwrap();
    });

    (base, handle)
}

#[tokio::test]
async fn office_ws_test_end_to_end_returns_receipt() {
    // 1) mock UBL
    let (ubl_base, _h1) = spawn_mock_ubl();

    // 2) Office apontando pro mock
    let (office_base, _h2) = spawn_office(ubl_base.clone());

    // 3) Chamar /office/ws/test
    let body = json!({
        "tenant": "acme",
        "workspace": "dan-42",
        "repo": "billing",
        "sha": "cafebabedeadbeef",
        "suite": "unit",
        "wait": true
    });

    let resp = reqwest::Client::new()
        .post(format!("{}/office/ws/test", office_base))
        .header("content-type", "application/json")
        .header("authorization", "Bearer sid-demo")
        .header("x-ubl-asc", "asc-demo")
        .body(body.to_string())
        .send().await.unwrap();

    assert_eq!(resp.status(), 202, "Office deve retornar 202 ACCEPTED");

    let json: Value = resp.json().await.unwrap();
    assert!(json.get("link_hash").is_some(), "deve retornar link_hash (atom_hash)");
    let receipt = json.get("receipt").expect("receipt esperado");
    assert_eq!(receipt.get("kind").unwrap(), "ws/receipt");
    assert_eq!(receipt.get("exit_code").unwrap(), 0);
}

kernel/rust/ubl-office/Cargo.toml (trecho de dev-deps)

[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread", "time"] }
axum = "0.7"
hyper = { version = "1", features = ["server"] }
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
serde_json = "1"

Rodar o teste:

cd kernel/rust/ubl-office
cargo test --test office_integration -- --nocapture

Você verá o Office subindo numa porta aleatória, o UBL “mock” noutra, a chamada POST /office/ws/test retornando 202 e o JSON com receipt coerente.

⸻

4) “Prova de Done” (rápida)
	•	cargo test -p ubl-office --test office_integration → PASSA e confirma 202 + receipt.
	•	clients/types/ubl/* importados no Messenger/SDK sem erro de tipo.
	•	Office segue chamando UBL_BASE/link/commit e depois .../ledger/{container}/latest até achar ws/receipt.trigger == atom_hash.

⸻

na próxima mensagem eu mando o patch “socket-unix”:
	•	como habilitar Postgres via /tmp/.s.PGSQL.5432,
	•	como fazer o UBL Server bindar em um Unix socket (ex.: /run/ubl/ubl-server.sock) e o Office falar com ele via reqwest com feature unix,
	•	e como ajustar systemd/pm2 + permissões (chmod 660, chown ubl:ubl) pra manter o “cano blindado”.perfeito — aqui vai o “pacotinho” que você pediu agora: types TS já gerados a partir dos três schemas (ws.test.request, ws.receipt, deploy.request) + teste de integração do Office que valida 202 + receipt (com UBL “mockado”). Na próxima mensagem eu mando o mesmo conjunto “o que mudar/adicionar” mas com a variação de Unix Socket (UBL/Office/Postgres sem TCP).

⸻

1) Tipos TypeScript (pré-gerados)

Coloque estes arquivos no repo:

clients/types/ubl/atoms/ws.test.request.d.ts
clients/types/ubl/atoms/ws.receipt.d.ts
clients/types/ubl/atoms/deploy.request.d.ts
clients/types/ubl/index.d.ts

clients/types/ubl/atoms/ws.test.request.d.ts

export interface WsLimits {
  cpu?: number;
  mem_mb?: number;
  timeout_sec?: number;
  net?: boolean;
}

export interface WsTestRequest {
  kind: "ws/test/request";
  tenant: string;
  workspace: string;
  repo: string;
  sha: string;
  suite: string;
  limits?: WsLimits;
}

clients/types/ubl/atoms/ws.receipt.d.ts

export interface ReceiptArtifact {
  name: string;
  hash: string;
}

export interface WsReceipt {
  kind: "ws/receipt";
  tenant: string;
  workspace: string;
  trigger: string;       // deve casar com o atom_hash do request
  exit_code: number;
  duration_ms: number;
  stdout_hash?: string;
  stderr_hash?: string;
  artifacts?: ReceiptArtifact[];
  toolchain?: Record<string, unknown>;
}

clients/types/ubl/atoms/deploy.request.d.ts

export type DeployStrategy = "canary" | "blue-green" | "rolling";
export type DeployEnv = "dev" | "stg" | "prod";

export interface DeployRequest {
  kind: "deploy/request";
  tenant: string;
  app: string;
  env: DeployEnv;
  image_digest: string;
  strategy: DeployStrategy;
  desired_replicas?: number;
}

clients/types/ubl/index.d.ts

export * from "./atoms/ws.test.request";
export * from "./atoms/ws.receipt";
export * from "./atoms/deploy.request";

Opcional (para automatizar geração no futuro):
npm i -D json-schema-to-typescript e um script schema:gen apontando contracts/ubl/atoms/*.schema.json → clients/types/ubl/atoms/*.d.ts.
Mas, por agora, os arquivos acima já estão prontos para uso.

⸻

2) Office pronto para teste de integração

Para facilitar o teste, exponha um builder de app no Office.

Novo kernel/rust/ubl-office/src/lib.rs

pub mod types;
pub mod asc;
pub mod routes { pub mod ws; pub mod deploy; }

use axum::{Router, routing::get};

pub fn app(ubl_base: String) -> Router {
    let ws_router = routes::ws::router(routes::ws::OfficeState { ubl_base: ubl_base.clone() });
    let deploy_router = routes::deploy::router(routes::deploy::OfficeState { ubl_base });

    Router::new()
        .merge(ws_router)
        .merge(deploy_router)
        .route("/health", get(|| async { axum::Json(serde_json::json!({"ok":true})) }))
}

Ajustar kernel/rust/ubl-office/src/main.rs

use axum::Router;

#[tokio::main]
async fn main() {
    let ubl_base = std::env::var("UBL_BASE").unwrap_or_else(|_| "http://localhost:8080".into());
    let app: Router = ubl_office::app(ubl_base);
    let addr = std::net::SocketAddr::from(([0,0,0,0], 8081));
    println!("Office listening on http://{addr}");
    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}

Nota: Mantém seu routes/ws.rs, routes/deploy.rs e types.rs iguais aos que definimos.

⸻

3) Teste de Integração (UBL “mockado” + Office real)

Arquivo de teste que sobe um UBL fake (Axum) e o Office real no mesmo runtime.
O UBL fake aceita POST /link/commit e, quando o Office fizer o poll do ledger, responde com um ws/receipt cujo trigger casa com o atom_hash do request.

Crie:

kernel/rust/ubl-office/tests/office_integration.rs

kernel/rust/ubl-office/tests/office_integration.rs

use axum::{Router, routing::{post, get}, Json, extract::Path};
use serde_json::{json, Value};
use std::{net::SocketAddr, sync::{Arc, Mutex}};
use tokio::task::JoinHandle;
use hyper::Server;

#[derive(Default, Clone)]
struct MockState {
    last_atom_hash: Arc<Mutex<Option<String>>>,
}

async fn mock_link_commit(
    axum::extract::State(state): axum::extract::State<MockState>,
    Json(payload): Json<Value>
) -> Json<Value> {
    // payload deve conter atom_hash (LinkCommit assinado pelo Office)
    let atom_hash = payload.get("atom_hash").and_then(|v| v.as_str()).unwrap_or_default().to_string();
    *state.last_atom_hash.lock().unwrap() = Some(atom_hash);
    Json(json!({"ok": true}))
}

async fn mock_ledger_latest(
    Path(container_id): Path<String>,
    axum::extract::State(state): axum::extract::State<MockState>,
) -> Json<Value> {
    let trigger = state.last_atom_hash.lock().unwrap().clone().unwrap_or_else(|| "none".into());
    // sempre devolve um recibo coerente
    let receipt = json!({
        "sequence": 42,
        "atom": {
            "kind": "ws/receipt",
            "tenant": "acme",
            "workspace": "dan-42",
            "trigger": trigger,
            "exit_code": 0,
            "duration_ms": 1234
        }
    });
    Json(Value::Array(vec![receipt]))
}

fn spawn_mock_ubl() -> (String, JoinHandle<()>) {
    let state = MockState::default();
    let app = Router::new()
        .route("/link/commit", post(mock_link_commit))
        .route("/ledger/:container_id/latest", get(mock_ledger_latest))
        .with_state(state);

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let base = format!("http://{}", addr);

    let handle = tokio::spawn(async move {
        Server::from_tcp(listener).unwrap().serve(app.into_make_service()).await.unwrap();
    });

    (base, handle)
}

fn spawn_office(ubl_base: String) -> (String, JoinHandle<()>) {
    let app = ubl_office::app(ubl_base);
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let base = format!("http://{}", addr);

    let handle = tokio::spawn(async move {
        Server::from_tcp(listener).unwrap().serve(app.into_make_service()).await.unwrap();
    });

    (base, handle)
}

#[tokio::test]
async fn office_ws_test_end_to_end_returns_receipt() {
    // 1) mock UBL
    let (ubl_base, _h1) = spawn_mock_ubl();

    // 2) Office apontando pro mock
    let (office_base, _h2) = spawn_office(ubl_base.clone());

    // 3) Chamar /office/ws/test
    let body = json!({
        "tenant": "acme",
        "workspace": "dan-42",
        "repo": "billing",
        "sha": "cafebabedeadbeef",
        "suite": "unit",
        "wait": true
    });

    let resp = reqwest::Client::new()
        .post(format!("{}/office/ws/test", office_base))
        .header("content-type", "application/json")
        .header("authorization", "Bearer sid-demo")
        .header("x-ubl-asc", "asc-demo")
        .body(body.to_string())
        .send().await.unwrap();

    assert_eq!(resp.status(), 202, "Office deve retornar 202 ACCEPTED");

    let json: Value = resp.json().await.unwrap();
    assert!(json.get("link_hash").is_some(), "deve retornar link_hash (atom_hash)");
    let receipt = json.get("receipt").expect("receipt esperado");
    assert_eq!(receipt.get("kind").unwrap(), "ws/receipt");
    assert_eq!(receipt.get("exit_code").unwrap(), 0);
}

kernel/rust/ubl-office/Cargo.toml (trecho de dev-deps)

[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread", "time"] }
axum = "0.7"
hyper = { version = "1", features = ["server"] }
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
serde_json = "1"

Rodar o teste:

cd kernel/rust/ubl-office
cargo test --test office_integration -- --nocapture

Você verá o Office subindo numa porta aleatória, o UBL “mock” noutra, a chamada POST /office/ws/test retornando 202 e o JSON com receipt coerente.

⸻

4) “Prova de Done” (rápida)
	•	cargo test -p ubl-office --test office_integration → PASSA e confirma 202 + receipt.
	•	clients/types/ubl/* importados no Messenger/SDK sem erro de tipo.
	•	Office segue chamando UBL_BASE/link/commit e depois .../ledger/{container}/latest até achar ws/receipt.trigger == atom_hash.

⸻

na próxima mensagem eu mando o patch “socket-unix”:
	•	como habilitar Postgres via /tmp/.s.PGSQL.5432,
	•	como fazer o UBL Server bindar em um Unix socket (ex.: /run/ubl/ubl-server.sock) e o Office falar com ele via reqwest com feature unix,
	•	e como ajustar systemd/pm2 + permissões (chmod 660, chown ubl:ubl) pra manter o “cano blindado”.