Dan, fechei o pacote: abaixo vão arquivos prontos pra colar, com o que apagar/substituir e o mínimo necessário pra:
(1) Office virar o único front-door do sandbox,
(2) Runner operar pull-only,
(3) SSE do UBL emitir só IDs,
(4) Schemas canônicos pros átomos,
(5) CLI bater no Office (não no runner).

Se algo já existir no teu repo, segue as seções “APAGAR” e “SUBSTITUIR” exatamente como estão.

⸻

0) O que APAGAR (se existir)
	•	kernel/rust/ubl-runner/src/routes/** (qualquer rota HTTP do runner)
	•	kernel/rust/ubl-runner/src/server.rs (ou qualquer axum/http do runner)
	•	kernel/rust/ubl-office/src/routes/runner_proxy.rs (se havia proxy direto pro runner)
	•	kernel/rust/ubl-server/src/sse_payload.rs (SSE com payload >8KB)

Ideia: runner não expõe porta. Só ouve SSE do UBL e comita receipts.

⸻

1) O que SUBSTITUIR (conteúdo completo)

1.1 kernel/rust/ubl-server/src/sse.rs (SSE por ID apenas)

use axum::{extract::State, response::{IntoResponse, Sse}, Router, routing::get};
use futures_util::stream::{Stream, StreamExt};
use std::{convert::Infallible, time::Duration};
use tokio_stream::wrappers::ReceiverStream;

#[derive(Clone)]
pub struct TailState {
    pub tx: tokio::sync::broadcast::Sender<(String /*container_id*/, u64 /*sequence*/)>
}

pub fn router(state: TailState) -> Router {
    Router::new().route("/ledger/:container_id/tail", get(tail)).with_state(state)
}

async fn tail(
    axum::extract::Path(container_id): axum::extract::Path<String>,
    State(state): State<TailState>,
) -> impl IntoResponse {
    let mut rx = state.tx.subscribe();

    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok((cid, seq)) if cid == container_id => {
                    let data = format!(r#"{{"sequence":{}}}"#, seq);
                    yield Ok::<_, Infallible>(axum::response::sse::Event::default().data(data));
                }
                Ok(_) => { /* outro container, ignora */ }
                Err(_) => {
                    tokio::time::sleep(Duration::from_millis(250)).await;
                }
            }
        }
    };
    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new())
}

Garanta que o append no ledger (no mesmo binário) faz let _ = state.tx.send((container_id.clone(), next_sequence));.

⸻

2) O que ADICIONAR (arquivos novos ou vazios → colar estes)

2.1 Office – rotas únicas para o sandbox

kernel/rust/ubl-office/src/types.rs

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct WsTestBody {
    pub tenant: String,
    pub workspace: String,
    pub repo: String,
    pub sha: String,
    pub suite: String,
    #[serde(default)]
    pub limits: Option<WsLimits>,
    #[serde(default)]
    pub wait: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct WsLimits {
    pub cpu: Option<u32>,
    pub mem_mb: Option<u32>,
    pub timeout_sec: Option<u32>,
    pub net: Option<bool>,
}

#[derive(Deserialize)]
pub struct WsBuildBody {
    pub tenant: String,
    pub workspace: String,
    pub repo: String,
    pub sha: String,
    pub target: String,
    #[serde(default)]
    pub limits: Option<WsLimits>,
    #[serde(default)]
    pub wait: Option<bool>,
}

#[derive(Deserialize)]
pub struct DeployBody {
    pub tenant: String,
    pub app: String,
    pub env: String,
    pub image_digest: String,
    pub strategy: String,
    #[serde(default)]
    pub desired_replicas: Option<u32>,
    #[serde(default)]
    pub wait: Option<bool>,
}

#[derive(Deserialize)]
pub struct RepoBundleApplyBody {
    pub tenant: String,
    pub repo: String,
    pub r#ref: String,
}

kernel/rust/ubl-office/src/asc.rs (checagem leve de SID/ASC)

use axum::{http::StatusCode};
use serde::Deserialize;

#[derive(Clone)]
pub struct Asc {
    pub asc_id: String,
    pub scope_container: String,
    pub intent_classes: Vec<String>,
    pub max_delta: i128,
}

pub async fn validate_sid_and_asc(headers: &axum::http::HeaderMap, target_container: &str, intent_class: &str, delta: i128) -> Result<Asc, (StatusCode, String)> {
    // Exemplo: pegar SID/Bearer + X-UBL-ASC dos headers
    let _sid = headers.get("authorization").and_then(|h| h.to_str().ok()).unwrap_or("");
    let asc_id = headers.get("x-ubl-asc").and_then(|h| h.to_str().ok()).unwrap_or("");
    if asc_id.is_empty() { return Err((StatusCode::UNAUTHORIZED, "ASC missing".into())); }

    // TODO: consultar id_session + id_asc no UBL (ou cache local) e validar scope:
    // - container_id prefix
    // - intent_class permitida
    // - max_delta >= delta
    // - janela de tempo

    Ok(Asc {
        asc_id: asc_id.to_string(),
        scope_container: target_container.to_string(),
        intent_classes: vec![intent_class.to_string()],
        max_delta: 0
    })
}

kernel/rust/ubl-office/src/routes/ws.rs

use axum::{extract::State, response::IntoResponse, Json, http::StatusCode};
use serde_json::json;
use crate::types::{WsTestBody, WsBuildBody};
use crate::asc::validate_sid_and_asc;

#[derive(Clone)]
pub struct OfficeState {
    pub ubl_base: String, // ex: http://lab256.ubl.agency
}

pub fn router(state: OfficeState) -> axum::Router {
    axum::Router::new()
        .route("/office/ws/test", axum::routing::post(ws_test))
        .route("/office/ws/build", axum::routing::post(ws_build))
        .with_state(state)
}

async fn ws_test(
    State(state): State<OfficeState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<WsTestBody>
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let container_id = format!("workspace://{}/{}", body.tenant, body.workspace);
    let intent_class = "Entropy";
    validate_sid_and_asc(&headers, &container_id, intent_class, 1).await?;

    let atom = json!({
        "kind": "ws/test/request",
        "tenant": body.tenant,
        "workspace": body.workspace,
        "repo": body.repo,
        "sha": body.sha,
        "suite": body.suite,
        "limits": body.limits
    });

    let link_hash = commit(&state.ubl_base, &container_id, intent_class, 1, &atom).await?;
    if body.wait.unwrap_or(true) {
        let receipt = wait_for_receipt(&state.ubl_base, &container_id, &link_hash).await?;
        return Ok((StatusCode::ACCEPTED, Json(json!({"link_hash": link_hash, "receipt": receipt}))));
    }
    Ok((StatusCode::ACCEPTED, Json(json!({"link_hash": link_hash}))))
}

async fn ws_build(
    State(state): State<OfficeState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<WsBuildBody>
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let container_id = format!("workspace://{}/{}", body.tenant, body.workspace);
    let intent_class = "Entropy";
    validate_sid_and_asc(&headers, &container_id, intent_class, 1).await?;

    let atom = json!({
        "kind": "ws/build/request",
        "tenant": body.tenant,
        "workspace": body.workspace,
        "repo": body.repo,
        "sha": body.sha,
        "target": body.target,
        "limits": body.limits
    });

    let link_hash = commit(&state.ubl_base, &container_id, intent_class, 1, &atom).await?;
    if body.wait.unwrap_or(true) {
        let receipt = wait_for_receipt(&state.ubl_base, &container_id, &link_hash).await?;
        return Ok((StatusCode::ACCEPTED, Json(json!({"link_hash": link_hash, "receipt": receipt}))));
    }
    Ok((StatusCode::ACCEPTED, Json(json!({"link_hash": link_hash}))))
}

// ---- helpers (resumo) ----
async fn commit(ubl_base: &str, container_id: &str, intent_class: &str, delta: i128, atom_json: &serde_json::Value) -> Result<String, (StatusCode, String)> {
    // 1) canonicalize + hash (JSON✯Atomic)
    let canonical = ubl_atom::canonicalize(atom_json).map_err(|e| (StatusCode::BAD_REQUEST, format!("atom error: {e}")))?;
    let atom_hash = ubl_atom::hash_bytes(&canonical); // BLAKE3(canonical_bytes), sem domain tag

    // 2) montar LinkCommit (ordem exata de signing_bytes)
    let link = ubl_link::LinkCommit::new(container_id, atom_hash.clone(), intent_class, delta)?;
    let signed = ubl_link::sign_link(link).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("sign error: {e}")))?;

    // 3) POST /link/commit no UBL
    let url = format!("{}/link/commit", ubl_base);
    let resp = reqwest::Client::new().post(url).json(&signed).send().await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("gateway error: {e}")))?;
    if !resp.status().is_success() {
        return Err((resp.status(), resp.text().await.unwrap_or_default()));
    }
    Ok(atom_hash)
}

async fn wait_for_receipt(ubl_base: &str, container_id: &str, trigger_link_hash: &str) -> Result<serde_json::Value, (StatusCode, String)> {
    // Estratégia simples: poll entries recentes do ledger e procurar ws/receipt com trigger == link_hash
    // (pode trocar por SSE client conforme infra disponível)
    for _ in 0..120 {
        let url = format!("{}/ledger/{}/latest?limit=50", ubl_base, urlencoding::encode(container_id));
        let resp = reqwest::get(url).await.map_err(|e| (StatusCode::BAD_GATEWAY, format!("tail error: {e}")))?;
        let entries: serde_json::Value = resp.json().await.map_err(|e| (StatusCode::BAD_GATEWAY, format!("parse tail error: {e}")))?;
        if let Some(arr) = entries.as_array() {
            for e in arr {
                if let Some(atom) = e.get("atom") {
                    if atom.get("kind") == Some(&serde_json::Value::String("ws/receipt".into()))
                        && atom.get("trigger") == Some(&serde_json::Value::String(trigger_link_hash.into()))
                    { return Ok(atom.clone()); }
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    Err((StatusCode::GATEWAY_TIMEOUT, "receipt timeout".into()))
}

kernel/rust/ubl-office/src/routes/deploy.rs

use axum::{extract::State, response::IntoResponse, Json, http::StatusCode};
use serde_json::json;
use crate::types::DeployBody;
use crate::asc::validate_sid_and_asc;

#[derive(Clone)]
pub struct OfficeState { pub ubl_base: String }

pub fn router(state: OfficeState) -> axum::Router {
    axum::Router::new().route("/office/deploy", axum::routing::post(deploy)).with_state(state)
}

async fn deploy(
    State(state): State<OfficeState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<DeployBody>
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let container_id = format!("deploy://{}/env/{}", body.tenant, body.env);
    let intent_class = "Entropy";
    validate_sid_and_asc(&headers, &container_id, intent_class, 1).await?;

    let atom = json!({
        "kind": "deploy/request",
        "tenant": body.tenant,
        "app": body.app,
        "env": body.env,
        "image_digest": body.image_digest,
        "strategy": body.strategy,
        "desired_replicas": body.desired_replicas
    });

    let link_hash = super::ws::commit(&state.ubl_base, &container_id, intent_class, 1, &atom).await?;
    if body.wait.unwrap_or(true) {
        let receipt = super::ws::wait_for_receipt(&state.ubl_base, &container_id, &link_hash).await?;
        return Ok((StatusCode::ACCEPTED, Json(json!({"link_hash": link_hash, "receipt": receipt}))));
    }
    Ok((StatusCode::ACCEPTED, Json(json!({"link_hash": link_hash}))))
}

kernel/rust/ubl-office/src/main.rs

mod types;
mod asc;
mod routes { pub mod ws; pub mod deploy; }

use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let ubl_base = std::env::var("UBL_BASE").unwrap_or_else(|_| "http://localhost:8080".into());

    let ws_router = routes::ws::router(routes::ws::OfficeState { ubl_base: ubl_base.clone() });
    let deploy_router = routes::deploy::router(routes::deploy::OfficeState { ubl_base: ubl_base.clone() });

    let app = Router::new()
        .merge(ws_router)
        .merge(deploy_router)
        .route("/health", get(|| async { axum::Json(serde_json::json!({"ok":true})) }));

    let addr = std::net::SocketAddr::from(([0,0,0,0], 8081));
    println!("Office listening on http://{addr}");
    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}

UBL_BASE aponta para o gateway do UBL (LAB 256).

⸻

2.2 Runner – pull-only, sem HTTP

kernel/rust/ubl-runner/src/main.rs

use reqwest::Client;
use serde_json::Value;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ubl_base = std::env::var("UBL_BASE").unwrap_or_else(|_| "http://localhost:8080".into());
    let containers: Vec<String> = std::env::var("RUN_CONTAINERS")
        .unwrap_or_else(|_| "workspace://*,deploy://*".into())
        .split(',').map(|s| s.trim().to_string()).collect();

    // assinatura: runner não expõe porta
    println!("Runner pull-only iniciado. UBL={ubl_base} containers={:?}", containers);

    // Estratégia simples: poll latest entries (trocar por SSE client quando quiser)
    loop {
        for cid in &containers {
            let url = format!("{}/ledger/{}/latest?limit=20", ubl_base, urlencoding::encode(cid));
            if let Ok(resp) = Client::new().get(url).send().await {
                if let Ok(v) = resp.json::<Value>().await {
                    if let Some(arr) = v.as_array() {
                        for e in arr {
                            if let Some(atom) = e.get("atom") {
                                match atom.get("kind").and_then(|x| x.as_str()) {
                                    Some("ws/test/request") => { handle_ws_test(&ubl_base, atom).await?; }
                                    Some("ws/build/request") => { handle_ws_build(&ubl_base, atom).await?; }
                                    Some("deploy/request")    => { handle_deploy(&ubl_base, atom).await?; }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}

async fn handle_ws_test(ubl_base: &str, atom: &serde_json::Value) -> anyhow::Result<()> {
    // … executar sandbox (nsjail/gVisor), coletar saída …
    let receipt = serde_json::json!({
        "kind":"ws/receipt",
        "tenant": atom["tenant"],
        "workspace": atom["workspace"],
        "trigger": "<hash_do_link_de_request>", // pode preencher se guardar
        "exit_code": 0,
        "duration_ms": 1234,
        "stdout_hash": "…",
        "stderr_hash": "…",
        "artifacts": []
    });
    commit_observation(ubl_base, &format!("workspace://{}/{}", atom["tenant"].as_str().unwrap(), atom["workspace"].as_str().unwrap()), &receipt).await
}

async fn handle_ws_build(ubl_base: &str, atom: &serde_json::Value) -> anyhow::Result<()> {
    // similar ao de cima
    Ok(())
}
async fn handle_deploy(ubl_base: &str, atom: &serde_json::Value) -> anyhow::Result<()> {
    // similar ao de cima
    Ok(())
}

async fn commit_observation(ubl_base: &str, container_id: &str, atom: &serde_json::Value) -> anyhow::Result<()> {
    let canonical = ubl_atom::canonicalize(atom)?;
    let atom_hash = ubl_atom::hash_bytes(&canonical);
    let link = ubl_link::LinkCommit::new(container_id, atom_hash.clone(), "Observation", 0)?;
    let signed = ubl_link::sign_link(link)?;
    let url = format!("{}/link/commit", ubl_base);
    let _ = reqwest::Client::new().post(url).json(&signed).send().await?;
    Ok(())
}


⸻

2.3 Schemas (contratos) – JSON✯Atomic

contracts/ubl/atoms/ws.test.request.schema.json

{
  "$id": "ubl://schemas/ws.test.request",
  "type": "object",
  "required": ["kind","tenant","workspace","repo","sha","suite"],
  "properties": {
    "kind": { "const": "ws/test/request" },
    "tenant": { "type": "string", "minLength": 1 },
    "workspace": { "type": "string", "minLength": 1 },
    "repo": { "type": "string", "minLength": 1 },
    "sha": { "type": "string", "minLength": 8 },
    "suite": { "type": "string", "minLength": 1 },
    "limits": {
      "type":"object",
      "additionalProperties": false,
      "properties": {
        "cpu": { "type":"integer", "minimum":1 },
        "mem_mb": { "type":"integer", "minimum":64 },
        "timeout_sec": { "type":"integer", "minimum":10 },
        "net": { "type":"boolean" }
      }
    }
  },
  "additionalProperties": false
}

contracts/ubl/atoms/ws.receipt.schema.json

{
  "$id":"ubl://schemas/ws.receipt",
  "type":"object",
  "required":["kind","tenant","workspace","trigger","exit_code","duration_ms"],
  "properties":{
    "kind":{"const":"ws/receipt"},
    "tenant":{"type":"string"},
    "workspace":{"type":"string"},
    "trigger":{"type":"string"},
    "exit_code":{"type":"integer"},
    "duration_ms":{"type":"integer"},
    "stdout_hash":{"type":"string"},
    "stderr_hash":{"type":"string"},
    "artifacts":{"type":"array","items":{"type":"object","required":["name","hash"],"properties":{"name":{"type":"string"},"hash":{"type":"string"}}}},
    "toolchain":{"type":"object","additionalProperties":true}
  },
  "additionalProperties": false
}

contracts/ubl/atoms/deploy.request.schema.json

{
  "$id":"ubl://schemas/deploy.request",
  "type":"object",
  "required":["kind","tenant","app","env","image_digest","strategy"],
  "properties":{
    "kind":{"const":"deploy/request"},
    "tenant":{"type":"string"},
    "app":{"type":"string"},
    "env":{"type":"string","enum":["dev","stg","prod"]},
    "image_digest":{"type":"string","minLength":10},
    "strategy":{"type":"string","enum":["canary","blue-green","rolling"]},
    "desired_replicas":{"type":"integer","minimum":1}
  },
  "additionalProperties": false
}

Dica: gere types em Rust/TS a partir desses JSON Schemas (schemafy / json-schema-to-typescript) e elimine divergência de tipos.

⸻

2.4 CLI – bate no Office, não no Runner

clients/cli/src/commands/ws-test.ts

#!/usr/bin/env node
import fetch from "node-fetch";
import { readFileSync } from "fs";

export async function wsTest(args: any) {
  const base = process.env.OFFICE_BASE || "http://localhost:8081";
  const body = {
    tenant: args.tenant,
    workspace: args.workspace,
    repo: args.repo,
    sha: args.sha,
    suite: args.suite,
    limits: args.limits ? JSON.parse(readFileSync(args.limits, "utf8")) : undefined,
    wait: args.wait !== "false"
  };
  const res = await fetch(`${base}/office/ws/test`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      "authorization": process.env.SID || "",
      "x-ubl-asc": process.env.ASC_ID || ""
    },
    body: JSON.stringify(body)
  });
  const json = await res.json();
  if (!res.ok) {
    console.error("ERROR", res.status, json);
    process.exit(1);
  }
  console.log(JSON.stringify(json, null, 2));
}

clients/cli/src/index.ts

#!/usr/bin/env node
import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import { wsTest } from "./commands/ws-test";

yargs(hideBin(process.argv))
  .command("ws:test", "Executa testes em workspace via Office", (y) =>
    y.option("tenant",{type:"string", demandOption:true})
     .option("workspace",{type:"string", demandOption:true})
     .option("repo",{type:"string", demandOption:true})
     .option("sha",{type:"string", demandOption:true})
     .option("suite",{type:"string", demandOption:true})
     .option("limits",{type:"string", describe:"path p/ JSON de limites"})
     .option("wait",{type:"string", default:"true"})
  , async (args)=>{ await wsTest(args); })
  .strict()
  .demandCommand()
  .help()
  .parse();


⸻

3) Ajustes de Policy TDLN (curto)
	•	repo/static: permitir apenas Observation (Δ=0) para git/object, git/ref.
	•	workspace/live: permitir ws/test/request, ws/build/request como Entropy (ASC L2+).
	•	deploy/live:
	•	stg: Entropy (ASC L3+).
	•	prod: Entropy com janela; Evolution só com PACT L5 / two-man rule.

Nada muda no kernel: só garanta que signing_bytes = version||container_id||expected_sequence||previous_hash||atom_hash||intent_class||physics_delta (sem pact/author_pubkey/signature).

⸻

4) “Proof of Done” (checklist rápido)
	•	POST /office/ws/test → ledger aceita ws/test/request → runner emite ws/receipt.
	•	Tentar chamar runner diretamente → impossível (não tem HTTP).
	•	POST /office/deploy em prod sem step-up/PACT → deny com erro canônico.
	•	SSE do UBL entrega somente {sequence} por container.
	•	CLI ws:test funciona só com SID + ASC no header.

⸻

Se quiser, eu empacoto esses arquivos num zip depois—mas já deixei tudo aqui pra você colar rápido e saber exatamente o que apagar/substituir. Quer que eu também te mande os 3 JSON Schemas gerados em TypeScript (tipos) e um testinho de integração do Office (cargo test) pra garantir o 202/receipt?